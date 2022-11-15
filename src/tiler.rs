use image::{imageops, DynamicImage, GenericImageView, ImageResult, RgbaImage, SubImage};
use rand::thread_rng;
use rand::Rng;
use std::fs::read_dir;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

/// Build and return an image from the given tiles.
pub fn process(lib_path: &str) -> IoResult<RgbaImage> {
    let lib_path_bufs = find_lib_images(lib_path)?;
    let lib_paths: Vec<&Path> = lib_path_bufs.iter().map(|f| f.as_path()).collect();

    let thumbnails = build_thumbnails(&lib_paths);

    let size = 1024;
    let output_image = build_output(&thumbnails, size);

    Ok(output_image)
}

// Path handling

fn find_lib_images(path: &str) -> IoResult<Vec<PathBuf>> {
    let path_reader = read_dir(path)?;
    let paths = path_reader.filter_map(|f| f.ok()).map(|f| f.path());
    Ok(paths.collect())
}

// Image handling

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

    let tile = extract_tile(&mut img?).to_image();

    let size = 256;
    let thumbnail = imageops::thumbnail(&tile, size, size);

    Ok(thumbnail)
}

/// Generate an output image from the given tiles
fn build_output(tiles: &[RgbaImage], size: u32) -> RgbaImage {
    let mut img = RgbaImage::new(size, size);

    randomly_pile_tiles_over(&mut img, tiles);

    img
}

/// Pile the given tiles over the target image
fn randomly_pile_tiles_over(target: &mut RgbaImage, tiles: &[RgbaImage]) {
    let (width, height) = target.dimensions();

    let min_tiles = 256;
    let tiles_to_place = min_tiles.max(tiles.len());

    let mut rng = thread_rng();
    let mut generate_random_coords = |w, h| {
        (
            rng.gen_range(-(w as i64)..width as i64),
            rng.gen_range(-(h as i64)..height as i64),
        )
    };

    for tile in tiles.iter().cycle().take(tiles_to_place) {
        let (w, h) = tile.dimensions();
        let (x, y) = generate_random_coords(w, h);

        imageops::overlay(target, tile, x, y);
    }
}

/// Load an image from a file
fn load_image(path: &Path) -> ImageResult<DynamicImage> {
    image::open(path)
}

/// Extract a square tile from the given image.
fn extract_tile(img: &mut DynamicImage) -> SubImage<&mut DynamicImage> {
    let (width, height) = img.dimensions();

    let (x, y, s) = if width < height {
        (0, (height - width) / 2, width)
    } else {
        ((width - height) / 2, 0, height)
    };

    imageops::crop(img, x, y, s, s)
}
