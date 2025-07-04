use std::collections::HashMap;

use image::{imageops, GenericImageView, RgbaImage};

use crate::analysis::{analyse, AnalysisOptions, ImageInfo};
use crate::core::{Dimensions, PixelRegion, Rectangle, TileLocation};

/// The strategy used to pick tiles for a given target.
pub trait TilingStrategy<T> {
    /// Choose the best set of tiles for this target image.
    fn choose(&self, target: &RgbaImage) -> Vec<TileLocation<T, PixelRegion>>;
}

pub struct IndependentStrategy<'a, T> {
    options: &'a AnalysisOptions,
    analysis: &'a HashMap<&'a T, ImageInfo>,
    cell_size: Dimensions,
}

impl<T> TilingStrategy<T> for IndependentStrategy<'_, T> {
    /// Choose the best set of tiles for this target image.
    fn choose(&self, target: &RgbaImage) -> Vec<TileLocation<T, PixelRegion>> {
        // This implementation assumes we can select the correct tile for
        // each cell independently.
        grid(target, &self.cell_size)
            .iter()
            .map(|t| (self.select_tile(target, t), PixelRegion::from(t)))
            .collect()
    }
}

impl<T> IndependentStrategy<'_, T> {
    pub fn new<'a>(
        analysis: &'a HashMap<&'a T, ImageInfo>,
        options: &'a AnalysisOptions,
        cell_size: Dimensions,
    ) -> IndependentStrategy<'a, T> {
        IndependentStrategy {
            options,
            analysis,
            cell_size,
        }
    }

    /// Choose the best tile for the given rectangle of the target.
    fn select_tile(&self, img: &RgbaImage, r: &Rectangle) -> &T {
        let target_info = analyse_cell(img, r, self.options);
        self.analysis
            .iter()
            .min_by_key(|(_, tile)| tile_difference_weight(&target_info, tile))
            .unwrap()
            .0
    }
}

/// Calculate the difference between the target region and a tile.
fn tile_difference_weight(target: &ImageInfo, tile: &ImageInfo) -> i32 {
    tile.diff(target).iter().sum::<i32>()
}

/// Build a grid of non-overlapping cell positions covering the target.
///
/// If the target isn't a perfect multiple of the cell_size in one or both
/// dimensions then the part not able to be covered is ignored.
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
mod strategy_tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    struct TestContext {
        analysis_options: AnalysisOptions,
        cell_size: Dimensions,
        blue_tile: RgbaImage,
        green_tile: RgbaImage,
    }

    fn setup() -> TestContext {
        TestContext {
            analysis_options: AnalysisOptions::new(Some(1)),
            cell_size: (10, 10),
            blue_tile: RgbaImage::from_pixel(10, 10, Rgba([0, 0, 255, 255])),
            green_tile: RgbaImage::from_pixel(10, 10, Rgba([0, 255, 0, 255])),
        }
    }

    fn analyse_tiles<'a>(
        ctx: &TestContext,
        tiles: Vec<&'a RgbaImage>,
    ) -> HashMap<&'a RgbaImage, ImageInfo> {
        tiles
            .into_iter()
            .map(|tile| (tile, analyse(tile, &ctx.analysis_options)))
            .collect()
    }

    #[test]
    fn test_independent_strategy_picks_best_match() {
        let ctx = setup();
        let target_image = RgbaImage::from_pixel(10, 10, Rgba([0, 0, 255, 255]));
        let analysis = analyse_tiles(&ctx, vec![&ctx.blue_tile, &ctx.green_tile]);
        let strategy = IndependentStrategy::new(&analysis, &ctx.analysis_options, ctx.cell_size);

        let result = strategy.choose(&target_image);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, &ctx.blue_tile);
    }

    #[test]
    fn test_independent_strategy_picks_best_match_for_each_cell() {
        let ctx = setup();
        let mut target_image = RgbaImage::from_pixel(20, 10, Rgba([0, 0, 255, 255]));
        let green_cell = RgbaImage::from_pixel(10, 10, Rgba([0, 255, 0, 255]));
        image::imageops::overlay(&mut target_image, &green_cell, 10, 0);
        let analysis = analyse_tiles(&ctx, vec![&ctx.blue_tile, &ctx.green_tile]);
        let strategy = IndependentStrategy::new(&analysis, &ctx.analysis_options, ctx.cell_size);

        let result = strategy.choose(&target_image);

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, &ctx.blue_tile);
        assert_eq!(result[1].0, &ctx.green_tile);
    }
}
