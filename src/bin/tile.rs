use std::env;
use tiler::{save, tile};

/// Create a tile from a source image
///
/// # Usage
///
/// tile <source_path> > tile.jpg
///
/// # Panics
///
/// Panics if source file path is not supplied as argument.
fn main() {
    let args: Vec<String> = env::args().collect();

    let Some(lib_path) = args.get(1) else {
        panic!("No target given")
    };
    let Ok(output_image) = tile(lib_path) else {
        panic!("Error converting")
    };
    let Ok(_) = save(&output_image, "/dev/stdout") else {
        panic!("Error saving")
    };
}
