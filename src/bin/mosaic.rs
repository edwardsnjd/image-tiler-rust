use std::env;
use tiler::{mosaic, save};

/// Create a mosaic
///
/// # Usage
///
/// mosaic <target> <tiles_dir> > output.jpg
///
/// # Panics
///
/// Panics if lib directory path is not supplied as argument.
fn main() {
    let args: Vec<String> = env::args().collect();

    let Some(target_path) = args.get(1) else {
        panic!("No target image path given")
    };
    let Some(lib_path) = args.get(2) else {
        panic!("No library images path given")
    };
    let Ok(output_image) = mosaic(target_path, lib_path) else {
        panic!("Error building")
    };
    let Ok(_) = save(&output_image, "/dev/stdout") else {
        panic!("Error saving")
    };
}
