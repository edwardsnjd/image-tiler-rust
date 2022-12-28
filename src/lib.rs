mod analysis;
mod core;
mod matching;
mod pile;
mod tiling;

use analysis::{analyse, ImageInfo};
use image::ImageFormat::Jpeg;
use image::{imageops, DynamicImage, GenericImageView, ImageResult, RgbaImage, SubImage};
use std::collections::HashMap;
use std::fs::read_dir;
use std::io::Result as IoResult;
use std::path::{Path, PathBuf};

use crate::analysis::AnalysisOptions;
use crate::core::{Dimensions, TileLocation};
use crate::matching::MatchingTileStrategy;
use crate::pile::RandomPileStrategy;
use crate::tiling::choose_tile_area;

// Public actions

/// Build and return a pile image from the given tiles.
pub fn pile(lib_path: &str) -> IoResult<RgbaImage> {
    let minimum_number_of_tiles = 64;
    let tile_size = (256, 256);
    let output_size = (4096, 4096);

    let lib_paths = find_lib_images(lib_path)?;
    let lib_images = load_available_images(&lib_paths);
    let thumbnails = build_thumbnails(&lib_images, tile_size);

    let strategy = RandomPileStrategy::new(&thumbnails, Some(minimum_number_of_tiles));
    let tiles = strategy.choose(output_size);

    let output_image = build_image(output_size, tiles);

    Ok(output_image)
}

/// Build and return a mosaic image from the given tiles.
pub fn mosaic(target_path: &str, lib_path: &str) -> IoResult<RgbaImage> {
    let analysis_size = 20;
    let cell_size = 20;
    let tile_size = 100;

    let target = load_image(Path::new(target_path)).unwrap();
    let lib_paths = find_lib_images(lib_path)?;

    let (width, height) = target.dimensions();
    let ratio = tile_size / cell_size;
    let ratio_i64 = ratio as i64;
    let output_size = (width * ratio, height * ratio);

    eprintln!("a: {}", analysis_size);
    eprintln!("c: {}", cell_size);
    eprintln!("t: {}", tile_size);
    eprintln!("r: x{}", ratio);
    eprintln!("s: ({},{})", width, height);
    eprintln!("o: ({},{})", output_size.0, output_size.1);

    let analysis_options = AnalysisOptions::new(Some(analysis_size));
    let lib_info = analyse_available_images(&lib_paths, &analysis_options);

    let strategy = MatchingTileStrategy::new(&lib_info, &analysis_options);
    let tiles = strategy.choose(&target, (cell_size, cell_size));

    let tiles = tiles
        .iter()
        .map(|(p, (x, y), (w, h))| (*p, (x * ratio_i64, y * ratio_i64), (w * ratio, h * ratio)))
        .collect();

    let output_image = build_image2(output_size, tiles);

    Ok(output_image)
}

/// Build and return a tile image from the given target.
pub fn tile(lib_path: &str) -> ImageResult<RgbaImage> {
    let size = (128, 128);
    load_image(Path::new(lib_path)).map(|img| build_tile(&img, size))
}

/// Save the given image as a JPEG
pub fn save(image: &RgbaImage, p: &str) -> ImageResult<()> {
    image.save_with_format(p, Jpeg)
}

// Path handling

fn find_lib_images(path: &str) -> IoResult<Vec<PathBuf>> {
    let path_reader = read_dir(path)?;
    let paths = path_reader.filter_map(|f| f.ok()).map(|f| f.path());
    Ok(paths.collect())
}

// Image handling
fn analyse_available_images<'a>(
    lib_paths: &'a [PathBuf],
    options: &'a AnalysisOptions,
) -> HashMap<&'a PathBuf, ImageInfo> {
    lib_paths
        .iter()
        .filter_map(|p| {
            load_image(p)
                .ok()
                .map(|i| analyse(&i, options))
                .map(|o| (p, o))
        })
        .collect()
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
    lib_images.iter().map(|img| build_tile(img, size)).collect()
}

/// Build a tile for the given image
fn build_tile(img: &RgbaImage, size: Dimensions) -> RgbaImage {
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

/// Build an image from tiles
fn build_image((width, height): Dimensions, tiles: Vec<TileLocation<RgbaImage>>) -> RgbaImage {
    let mut output = RgbaImage::new(width, height);
    for (tile, (x, y), (_w, _h)) in tiles {
        imageops::overlay(&mut output, tile, x, y);
    }
    output
}

fn build_image2((width, height): Dimensions, tiles: Vec<TileLocation<PathBuf>>) -> RgbaImage {
    let mut output = RgbaImage::new(width, height);
    for (tile, (x, y), (w, h)) in tiles {
        let img = load_image(tile).unwrap();
        let thumb = imageops::thumbnail(&img, w, h);
        imageops::overlay(&mut output, &thumb, x, y);
    }
    output
}
