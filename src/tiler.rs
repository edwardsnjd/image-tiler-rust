use image::{imageops, DynamicImage, ImageResult, RgbaImage};
use std::fs::read_dir;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

/// Size for generated thumbnails (square)
const THUMBNAIL_SIZE: u32 = 256;

/// Size of generated output image (square)
const OUTPUT_SIZE: u32 = 1024;

/// Build and return an image from the given tiles.
pub fn process(lib_path: &str) -> IoResult<RgbaImage> {
    let lib_path_bufs = find_lib_images(lib_path)?;
    let lib_paths: Vec<&Path> = lib_path_bufs.iter().map(|f| f.as_path()).collect();

    let thumbnails = build_thumbnails(&lib_paths);
    let s = strategy::random_pile_strategy(&thumbnails);

    let output_image = build_output(&s, OUTPUT_SIZE);

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
    let mut img = RgbaImage::new(size, size);

    for (tile, x, y) in tile_strategy.choose(&img) {
        imageops::overlay(&mut img, tile, x, y);
    }

    img
}

/// Load an image from a file
fn load_image(path: &Path) -> ImageResult<DynamicImage> {
    image::open(path)
}

// Thumbnails

/// Build thumbnails for the images at the given paths
fn build_thumbnails(paths: &[&Path]) -> Vec<RgbaImage> {
    paths
        .iter()
        .filter_map(|path| build_thumbnail(path).ok())
        .collect()
}

/// Try to build a thumbnail for the given path
fn build_thumbnail(path: &Path) -> ImageResult<RgbaImage> {
    let img = load_image(path);

    let tile = tiling::extract_tile(&mut img?).to_image();

    let thumbnail = imageops::thumbnail(&tile, THUMBNAIL_SIZE, THUMBNAIL_SIZE);

    Ok(thumbnail)
}

/// Strategies for tiling an image
mod strategy {
    use image::RgbaImage;
    use rand::thread_rng;
    use rand::Rng;

    /// Minimum number of tiles to draw (repeat tiles if fewer than this)
    const MIN_TILES: usize = 128;

    /// The main trait capturing the idea of a strategy to choose tiles
    pub trait TileStrategy {
        fn choose(&self, target: &RgbaImage) -> Vec<TileLocation<RgbaImage>>;
    }

    /// Convenience type alias for a tile and where to draw it
    pub type TileLocation<'a, T> = (&'a T, i64, i64);

    // Random pile

    pub struct RandomPileStrategy<'a> {
        tiles: &'a [RgbaImage],
    }

    pub fn random_pile_strategy(tiles: &[RgbaImage]) -> RandomPileStrategy {
        RandomPileStrategy { tiles }
    }

    impl<'a> TileStrategy for RandomPileStrategy<'a> {
        fn choose(&self, target: &RgbaImage) -> Vec<TileLocation<RgbaImage>> {
            let (width, height) = target.dimensions();
            random_pile(self.tiles, width, height)
        }
    }

    /// Trait capturing any dimensioned structure
    trait Dimensioned {
        fn dimensions(&self) -> (u32, u32);
    }

    /// Explicitly treat images as dimensioned
    impl Dimensioned for RgbaImage {
        fn dimensions(&self) -> (u32, u32) {
            self.dimensions()
        }
    }

    /// Place tiles in a random pile
    fn random_pile<T>(tiles: &[T], width: u32, height: u32) -> Vec<TileLocation<T>>
    where
        T: Dimensioned,
    {
        let tiles_to_place = MIN_TILES.max(tiles.len());

        let mut rng = thread_rng();
        let mut generate_random_coords = |w, h| {
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
                let (w, h) = tile.dimensions();
                let (x, y) = generate_random_coords(w, h);
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
            fn dimensions(&self) -> (u32, u32) {
                (self.width, self.height)
            }
        }

        #[test]
        fn test_returns_zero_tiles_for_no_input() {
            let tiles: Vec<FakeImage> = vec![];
            let actual = random_pile(&tiles, 100, 200);
            assert_eq!(actual.len(), 0);
        }

        #[test]
        fn test_returns_minimum_number_even_if_insufficient_tiles() {
            let tiles = vec![fake_image(10, 10)];
            let actual = random_pile(&tiles, 100, 200);
            assert_eq!(actual.len(), MIN_TILES);
        }

        #[test]
        fn test_all_coords_in_bounds() {
            let (width, height): (u32, u32) = (100, 200);
            let tile_size: u32 = 10;
            let tiles = vec![fake_image(tile_size, tile_size)];

            let actual = random_pile(&tiles, width, height);

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
    use image::{imageops, DynamicImage, GenericImageView, SubImage};

    /// Extract a square tile from the given image.
    pub fn extract_tile(img: &mut DynamicImage) -> SubImage<&mut DynamicImage> {
        let (width, height) = img.dimensions();
        let tile = choose_tile_area(width, height);
        imageops::crop(img, tile.x, tile.y, tile.width, tile.height)
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
