use image::{imageops, DynamicImage, ImageResult, RgbaImage};
use std::fs::read_dir;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

/// Size for generated thumbnails (square)
const THUMBNAIL_SIZE: u32 = 256;

/// Size of generated output image (square)
const OUTPUT_SIZE: u32 = 1024;

/// Alias for width and height
type Dimensions = (u32, u32);

/// Build and return an image from the given tiles.
pub fn process(lib_path: &str) -> IoResult<RgbaImage> {
    let lib_paths = find_lib_images(lib_path)?;
    let lib_images = load_available_images(&lib_paths);
    let tiles = build_thumbnails(&lib_images, (THUMBNAIL_SIZE, THUMBNAIL_SIZE));

    let strategy = strategy::random_pile_strategy(&tiles, Some(4));

    let output_image = build_output(&strategy, OUTPUT_SIZE);

    Ok(output_image)
}

// Path handling

fn find_lib_images(path: &str) -> IoResult<Vec<PathBuf>> {
    let path_reader = read_dir(path)?;
    let paths = path_reader.filter_map(|f| f.ok()).map(|f| f.path());
    Ok(paths.collect())
}

// Image handling

/// Generate an output image from the given tiles
fn build_output(tile_strategy: &dyn strategy::TileStrategy, size: u32) -> RgbaImage {
    let mut output = RgbaImage::new(size, size);

    for (tile, x, y) in tile_strategy.choose(output.dimensions()) {
        imageops::overlay(&mut output, tile, x, y);
    }

    output
}

fn load_available_images(lib_paths: &[PathBuf]) -> Vec<RgbaImage> {
    lib_paths
        .iter()
        .filter_map(|p| load_image(p).ok())
        .collect()
}

/// Load an image from a file
fn load_image(path: &Path) -> ImageResult<RgbaImage> {
    image::open(path).map(DynamicImage::into_rgba8)
}

// Thumbnails

fn build_thumbnails(lib_images: &[RgbaImage], size: Dimensions) -> Vec<RgbaImage> {
    lib_images
        .iter()
        .map(|img| build_thumbnail(img, size))
        .collect()
}

/// Build a thumbnail for the given image
fn build_thumbnail(img: &RgbaImage, size: Dimensions) -> RgbaImage {
    let (width, height) = size;
    let tile = tiling::extract_tile(img).to_image();
    imageops::thumbnail(&tile, width, height)
}

/// Strategies for tiling an image
mod strategy {
    use super::Dimensions;
    use image::RgbaImage;
    use rand::thread_rng;
    use rand::Rng;

    /// Minimum number of tiles to draw (repeat tiles if fewer than this)
    const MIN_TILES: usize = 128;

    /// The main trait capturing the idea of a strategy to choose tiles
    pub trait TileStrategy {
        fn choose(&self, size: Dimensions) -> Vec<TileLocation<RgbaImage>>;
    }

    /// Convenience type alias for a tile and where to draw it
    pub type TileLocation<'a, T> = (&'a T, i64, i64);

    // Random pile

    pub struct RandomPileStrategy<'a> {
        tiles: &'a [RgbaImage],
        min_tiles: usize,
    }

    pub fn random_pile_strategy(tiles: &[RgbaImage], min_tiles: Option<usize>) -> RandomPileStrategy {
        let min_tiles = min_tiles.unwrap_or(MIN_TILES);
        RandomPileStrategy { tiles, min_tiles }
    }

    impl TileStrategy for RandomPileStrategy<'_> {
        fn choose(&self, size: Dimensions) -> Vec<TileLocation<RgbaImage>> {
            random_pile(&self.tiles, self.min_tiles, size)
        }
    }

    /// Trait capturing any dimensioned structure
    trait Dimensioned {
        fn dimensions(&self) -> Dimensions;
    }

    /// Explicitly treat images as dimensioned
    impl Dimensioned for RgbaImage {
        fn dimensions(&self) -> Dimensions {
            self.dimensions()
        }
    }

    /// Place tiles in a random pile
    fn random_pile<T>(tiles: &[T], min_tiles: usize, size: Dimensions) -> Vec<TileLocation<T>>
    where
        T: Dimensioned,
    {
        let tiles_to_place = min_tiles.max(tiles.len());
        let (width, height) = size;

        let mut rng = thread_rng();
        let mut generate_random_coords = |(w, h)| {
            (
                rng.gen_range(-(w as i64)..width as i64),
                rng.gen_range(-(h as i64)..height as i64),
            )
        };

        tiles
            .iter()
            .cycle()
            .take(tiles_to_place)
            .map(|tile| {
                let size = tile.dimensions();
                let (x, y) = generate_random_coords(size);
                (tile, x, y)
            })
            .collect()
    }

    #[cfg(test)]
    mod test {
        use super::*;

        struct FakeImage {
            width: u32,
            height: u32,
        }

        fn fake_image(width: u32, height: u32) -> FakeImage {
            FakeImage { width, height }
        }

        impl Dimensioned for FakeImage {
            fn dimensions(&self) -> Dimensions {
                (self.width, self.height)
            }
        }

        #[test]
        fn test_returns_zero_tiles_for_no_input() {
            let tiles: Vec<FakeImage> = vec![];
            let actual = random_pile(&tiles, 2, (100, 200));
            assert_eq!(actual.len(), 0);
        }

        #[test]
        fn test_returns_minimum_number_even_if_insufficient_tiles() {
            let tiles = vec![fake_image(10, 10)];
            let actual = random_pile(&tiles, 7, (100, 200));
            assert_eq!(actual.len(), 7);
        }

        #[test]
        fn test_all_coords_in_bounds() {
            let (width, height): Dimensions = (100, 200);
            let tile_size: u32 = 10;
            let tiles = vec![fake_image(tile_size, tile_size)];

            let actual = random_pile(&tiles, 7, (width, height));

            let xcoords: Vec<i64> = actual.iter().map(|loc| loc.1).collect();
            let ycoords: Vec<i64> = actual.iter().map(|loc| loc.2).collect();

            let all_x_valid = xcoords
                .iter()
                .all(|x| -1 * tile_size as i64 <= *x && *x < width as i64);
            let all_y_valid = ycoords
                .iter()
                .all(|y| -1 * tile_size as i64 <= *y && *y < height as i64);

            assert_eq!(all_x_valid, true, "{:?}", xcoords);
            assert_eq!(all_y_valid, true, "{:?}", ycoords);
        }
    }
}

/// Extracting tiles from an image
mod tiling {
    use image::{imageops, GenericImageView, SubImage};

    /// Extract a square tile from the given image.
    pub fn extract_tile<I>(img: &I) -> SubImage<&I>
    where
        I: GenericImageView,
    {
        let (width, height) = img.dimensions();
        let tile = choose_tile_area(width, height);
        imageops::crop_imm(img, tile.x, tile.y, tile.width, tile.height)
    }

    #[derive(Eq, PartialEq, Debug)]
    struct Rectangle {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    }

    fn rectangle(x: u32, y: u32, width: u32, height: u32) -> Rectangle {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }

    /// Choose the area to use as a tile from an image of the given dimensions.
    fn choose_tile_area(width: u32, height: u32) -> Rectangle {
        let (x, y, s) = if width < height {
            (0, (height - width) / 2, width)
        } else {
            ((width - height) / 2, 0, height)
        };

        rectangle(x, y, s, s)
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_chooses_central_square_for_portrait_tile() {
            assert_eq!(choose_tile_area(10, 20), rectangle(0, 5, 10, 10));
        }

        #[test]
        fn test_chooses_central_square_for_landscape_tile() {
            assert_eq!(choose_tile_area(20, 10), rectangle(5, 0, 10, 10));
        }
    }
}

/// Analysis of images
mod analysis {
    use core::fmt::Debug;

    use image::{imageops, Pixel, RgbaImage};

    #[allow(dead_code)]
    const SAMPLE_SIZE: u8 = 8;

    #[allow(dead_code)]
    pub fn analyse(img: &RgbaImage, options: &AnalysisOptions) -> ImageInfo {
        let size = options.sample_size as u32;
        let (width, height) = img.dimensions();

        let foo = imageops::thumbnail(img, size, size);

        let colors = foo
            .pixels()
            .map(|p| {
                let vals = p.channels();
                ColorInfo {
                    red: vals[0].to_owned(),
                    blue: vals[1].to_owned(),
                    green: vals[2].to_owned(),
                }
            })
            .collect();

        ImageInfo {
            width,
            height,
            colors,
        }
    }

    pub struct AnalysisOptions {
        sample_size: u8,
    }

    #[allow(dead_code)]
    pub fn options(sample_size: Option<u8>) -> AnalysisOptions {
        AnalysisOptions {
            sample_size: sample_size.unwrap_or(SAMPLE_SIZE),
        }
    }

    /// Data describing the image, suitable for comparison between images.
    #[derive(Debug, PartialEq, Eq)]
    pub struct ImageInfo {
        width: u32,
        height: u32,
        colors: Vec<ColorInfo>,
    }

    /// Data describing the color of a pixel.
    #[derive(PartialEq, Eq)]
    pub struct ColorInfo {
        red: u8,
        blue: u8,
        green: u8,
    }

    impl Debug for ColorInfo {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "({},{},{})", self.red, self.blue, self.green)
        }
    }

    #[cfg(test)]
    mod test {
        use super::*;

        const BLACK: ColorInfo = ColorInfo {
            red: 0,
            blue: 0,
            green: 0,
        };

        #[test]
        fn test_foo() {
            let size = 100;
            let img = RgbaImage::new(size, size);

            let opts = options(Some(1));

            let result = analyse(&img, &opts);

            assert_eq!(
                result,
                ImageInfo {
                    width: size,
                    height: size,
                    colors: vec![BLACK],
                }
            );
        }
    }
}
