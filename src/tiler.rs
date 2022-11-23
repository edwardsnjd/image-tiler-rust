use image::{ImageResult, RgbaImage, imageops, GenericImageView, SubImage, DynamicImage};
use std::fs::read_dir;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use crate::core::{Dimensions, build_output};
use crate::pile::random_pile_strategy;
use crate::tiling::choose_tile_area;

/// Size for generated thumbnails (square)
const THUMBNAIL_SIZE: u32 = 256;

/// Size of generated output image (square)
const OUTPUT_SIZE: u32 = 1024;

/// Build and return an image from the given tiles.
pub fn process(lib_path: &str) -> IoResult<RgbaImage> {
    let lib_paths = find_lib_images(lib_path)?;
    let lib_images = load_available_images(&lib_paths);

    let tiles = build_thumbnails(&lib_images, (THUMBNAIL_SIZE, THUMBNAIL_SIZE));

    let strategy = random_pile_strategy(&tiles, Some(4));

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
    let tile = extract_tile(img).to_image();
    imageops::thumbnail(&tile, width, height)
}

/// Extract a square tile from the given image.
fn extract_tile<I>(img: &I) -> SubImage<&I>
where
    I: GenericImageView,
{
    let (width, height) = img.dimensions();
    let tile = choose_tile_area(width, height);
    imageops::crop_imm(img, tile.x, tile.y, tile.width, tile.height)
}
