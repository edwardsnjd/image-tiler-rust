use std::collections::HashMap;

use image::{imageops, GenericImageView, RgbaImage};

use crate::analysis::{analyse, AnalysisOptions, ImageInfo};
use crate::core::{Dimensions, PixelRegion, Rectangle, TileLocation};

pub struct MatchingTileStrategy<'a, T> {
    options: &'a AnalysisOptions,
    analysis: &'a HashMap<&'a T, ImageInfo>,
}

impl<T> MatchingTileStrategy<'_, T> {
    pub fn new<'a>(
        analysis: &'a HashMap<&T, ImageInfo>,
        options: &'a AnalysisOptions,
    ) -> MatchingTileStrategy<'a, T> {
        MatchingTileStrategy { options, analysis }
    }

    // Independent tile selection

    pub fn choose(
        &self,
        target: &RgbaImage,
        cell_size: &Dimensions,
    ) -> Vec<TileLocation<T, PixelRegion>> {
        // This implementation assumes we can select the correct tile for
        // each cell independently.
        grid(target, cell_size)
            .iter()
            .map(|t| self.select_tile(target, t))
            .collect()
    }

    fn select_tile(&self, img: &RgbaImage, r: &Rectangle) -> TileLocation<T, PixelRegion> {
        let target_info = analyse_cell(img, r, &self.options);
        let best_tile = *self
            .analysis
            .iter()
            .min_by_key(|(_, info)| info.diff(&target_info).iter().sum::<i32>())
            .unwrap()
            .0;
        (best_tile, PixelRegion::from(r))
    }
}

fn grid<I>(target: &I, cell_size: &Dimensions) -> Vec<Rectangle>
where
    I: GenericImageView,
{
    let (tw, th) = target.dimensions();
    let (cw, ch) = cell_size;

    let xs = (0..tw).step_by(*cw as usize);
    let ys = (0..th).step_by(*ch as usize);

    itertools::iproduct!(xs, ys)
        .map(|(x, y)| Rectangle::new(x, y, *cw, *ch))
        .collect()
}

fn analyse_cell(img: &RgbaImage, r: &Rectangle, options: &AnalysisOptions) -> ImageInfo {
    let target = imageops::crop_imm(img, r.x, r.y, r.width, r.height);
    analyse(&target.to_image(), options)
}

#[cfg(test)]
mod test {
    // use super::*;

    // #[test]
    // fn test_foo() {
    //     todo!();
    // }
}
