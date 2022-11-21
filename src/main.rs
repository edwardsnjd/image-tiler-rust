mod tiler;
mod analysis;

use image::ImageError;
use image::ImageFormat::Jpeg;
use std::env;

pub use crate::tiler::process;

/// Wrap main process in CLI app
///
/// # Usage
///
/// tiler <tiles_dir> > output.jpg
///
/// # Panics
///
/// Panics if lib directory path is not supplied as argument.
fn main() -> Result<(), ImageError> {
    let args: Vec<String> = env::args().collect();
    let lib_path = args.get(1).unwrap();

    let output_image = process(lib_path)?;

    output_image.save_with_format(&"/dev/stdout", Jpeg)
}
