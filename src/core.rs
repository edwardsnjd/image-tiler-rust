use image::{RgbaImage, imageops};

/// Alias for width and height
pub type Dimensions = (u32, u32);

/// Convenience type alias for a tile and where to draw it
pub type TileLocation<'a, T> = (&'a T, i64, i64);

/// The main trait capturing the idea of a strategy to choose tiles
pub trait TileStrategy {
    fn choose(&self, target: &RgbaImage) -> Vec<TileLocation<RgbaImage>>;
}

/// Generate an output image from the given tiles
pub fn build_output(tile_strategy: &dyn TileStrategy, size: u32) -> RgbaImage {
    let mut output = RgbaImage::new(size, size);

    for (tile, x, y) in tile_strategy.choose(&output) {
        imageops::overlay(&mut output, tile, x, y);
    }

    output
}
