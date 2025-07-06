use image::{imageops, GenericImageView, RgbaImage};
use std::cmp::max;
use std::collections::HashMap;

use crate::analysis::{analyse, AnalysisOptions, ImageInfo};
use crate::core::{Dimensions, PixelRegion, Rectangle, TileLocation};

/// The strategy used to pick tiles for a given target.
pub trait TilingStrategy<T> {
    /// Choose the best set of tiles for this target image.
    fn choose(&self, target: &RgbaImage) -> Vec<TileLocation<T, PixelRegion>>;
}

// Independent tile selection

pub struct IndependentStrategy<'a, T> {
    options: &'a AnalysisOptions,
    analysis: &'a HashMap<&'a T, ImageInfo>,
    cell_size: Dimensions,
}

impl<T> TilingStrategy<T> for IndependentStrategy<'_, T> {
    /// Choose the best set of tiles for this target image.
    ///
    /// This picks the best tile independent of all other tiles.
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
    #[allow(dead_code)]
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

// Holistic tile selection

pub struct HolisticStrategy<'a, T, U>
where
    T: Eq + std::hash::Hash,
    U: Fn(i32) -> i32,
{
    options: &'a AnalysisOptions,
    analysis: &'a HashMap<&'a T, ImageInfo>,
    cell_size: Dimensions,
    duplicate_penalty: U,
}

#[allow(dead_code)]
impl<'a, T, U> HolisticStrategy<'a, T, U>
where
    T: Eq + std::hash::Hash,
    U: Fn(i32) -> i32,
{
    pub fn new(
        analysis: &'a HashMap<&'a T, ImageInfo>,
        options: &'a AnalysisOptions,
        cell_size: Dimensions,
        duplicate_penalty: U,
    ) -> Self {
        Self {
            options,
            analysis,
            cell_size,
            duplicate_penalty,
        }
    }

    /// Choose the best tile for the given rectangle of the target.
    fn evaluate_tile(&self, img: &RgbaImage, r: &Rectangle) -> HashMap<&'a T, i32> {
        let target_info = analyse_cell(img, r, self.options);
        self.analysis
            .iter()
            .map(|(i, tile)| (*i, tile_difference_weight(&target_info, tile)))
            .collect()
    }
}

impl<T, U> TilingStrategy<T> for HolisticStrategy<'_, T, U>
where
    T: Eq + std::hash::Hash,
    U: Fn(i32) -> i32,
{
    /// Choose the best set of tiles for this target image.
    ///
    /// This aims to avoid duplicates by penalising duplicates.
    fn choose(&self, target: &RgbaImage) -> Vec<TileLocation<T, PixelRegion>> {
        let rects = grid(target, &self.cell_size);

        // Use the normal difference to evaluate the cost of each library image for each tile
        let mut cell_options: HashMap<&Rectangle, HashMap<&T, i32>> = rects
            .iter()
            .map(|rect| (rect, self.evaluate_tile(target, rect)))
            .collect();

        adjust_weights(&mut cell_options, &rects, &self.duplicate_penalty);

        // Pick the best image for each tile
        cell_options
            .iter()
            .map(|(rect, lib_weights)| {
                let (best, _) = lib_weights
                    .iter()
                    .min_by_key(|(_, weight)| *weight)
                    .unwrap();
                (rect, best)
            })
            .map(|(rect, &best)| (best, PixelRegion::from(rect)))
            .collect()
    }
}

// Utilities

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

/// Analyse part of a target image.
fn analyse_cell(img: &RgbaImage, r: &Rectangle, options: &AnalysisOptions) -> ImageInfo {
    let target = imageops::crop_imm(img, r.x, r.y, r.width, r.height);
    analyse(&target.to_image(), options)
}

fn adjust_weights<T, U>(
    cell_options: &mut HashMap<&Rectangle, HashMap<&T, i32>>,
    rects: &[Rectangle],
    duplicate_penalty: &U,
) where
    T: Eq + std::hash::Hash,
    U: Fn(i32) -> i32,
{
    // Find the best tile for each rectangle
    // TODO: Should order matter?
    for rect in rects.iter() {
        // Find best tile for this rect...
        let hash_map = cell_options.get(&rect).unwrap();
        let min_by_key = hash_map.iter().min_by_key(|(_, weight)| *weight).unwrap();
        let best_tile = *min_by_key.0;

        // Penalise this tile in all following rectangles
        let following_rects = rects.iter().skip_while(|&r| r != rect).skip(1);
        for following_rect in following_rects {
            let lib_weights = cell_options.get_mut(following_rect).unwrap();
            let weight = lib_weights.get_mut(best_tile).unwrap();

            let dist = num::abs(following_rect.y as i32 - rect.y as i32)
                + num::abs(following_rect.x as i32 - rect.x as i32);
            let penalty = (duplicate_penalty)(dist);

            *weight += penalty;
        }
    }
}

pub fn penalty_by_distance(analysis_size: u8, dist_threshold: u32) -> impl Fn(i32) -> i32 {
    let analysis_size = analysis_size as i32;
    let dist_threshold = dist_threshold as i32;
    let max_penalty = 255 * 255 * 3 * analysis_size * analysis_size / 20;

    move |dist: i32| {
        let penalty = (max_penalty / dist_threshold) * (dist_threshold - dist);
        max(0, penalty)
    }
}

// Tests

#[cfg(test)]
mod strategy_tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use itertools::Itertools;

    // Utils

    struct TestContext {
        analysis_options: AnalysisOptions,
        cell_size: Dimensions,
        red: Rgba<u8>,
        green: Rgba<u8>,
        blue: Rgba<u8>,
        red_tile1: RgbaImage,
        red_tile2: RgbaImage,
        red_tile3: RgbaImage,
        green_tile: RgbaImage,
        blue_tile: RgbaImage,
    }

    fn setup() -> TestContext {
        let red_pixel = Rgba([255, 0, 0, 255]);
        let redish_pixel = Rgba([254, 0, 0, 255]);
        let redy_pixel = Rgba([253, 0, 0, 255]);
        let green_pixel = Rgba([0, 255, 0, 255]);
        let blue_pixel = Rgba([0, 0, 255, 255]);

        TestContext {
            analysis_options: AnalysisOptions::new(Some(1)),
            cell_size: (10, 10),
            red: red_pixel,
            green: green_pixel,
            blue: blue_pixel,
            red_tile1: RgbaImage::from_pixel(10, 10, red_pixel),
            red_tile2: RgbaImage::from_pixel(10, 10, redish_pixel),
            red_tile3: RgbaImage::from_pixel(10, 10, redy_pixel),
            green_tile: RgbaImage::from_pixel(10, 10, green_pixel),
            blue_tile: RgbaImage::from_pixel(10, 10, blue_pixel),
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

    fn sort_by_position<'a, T>(
        result: &'a Vec<(&'a T, PixelRegion)>,
    ) -> Vec<&'a (&'a T, PixelRegion)> {
        result
            .iter()
            .sorted_by(|a, b| Ord::cmp(&a.1, &b.1))
            .collect()
    }

    fn blue_image(ctx: &TestContext) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
        RgbaImage::from_pixel(10, 10, ctx.blue)
    }

    fn blue_green_image(ctx: &TestContext) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
        let mut img = RgbaImage::from_pixel(20, 10, ctx.blue);
        let green_cell = RgbaImage::from_pixel(10, 10, ctx.green);
        image::imageops::overlay(&mut img, &green_cell, 10, 0);
        img
    }

    fn red_image(ctx: &TestContext) -> image::ImageBuffer<Rgba<u8>, Vec<u8>> {
        RgbaImage::from_pixel(30, 10, ctx.red)
    }

    mod independent_strategy {
        use crate::strategy::strategy_tests::*;

        #[test]
        fn test_independent_strategy_picks_best_match() {
            let ctx = setup();

            let blue_image = blue_image(&ctx);
            let analysis = analyse_tiles(&ctx, vec![&ctx.blue_tile, &ctx.green_tile]);
            let strategy =
                IndependentStrategy::new(&analysis, &ctx.analysis_options, ctx.cell_size);

            let result = strategy.choose(&blue_image);

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, &ctx.blue_tile);
        }

        #[test]
        fn test_independent_strategy_picks_best_match_for_each_cell() {
            let ctx = setup();

            let blue_green_image = blue_green_image(&ctx);
            let analysis = analyse_tiles(&ctx, vec![&ctx.blue_tile, &ctx.green_tile]);
            let strategy =
                IndependentStrategy::new(&analysis, &ctx.analysis_options, ctx.cell_size);

            let result = strategy.choose(&blue_green_image);

            assert_eq!(result.len(), 2);
            let result = sort_by_position(&result);
            assert_eq!(result[0].0, &ctx.blue_tile);
            assert_eq!(result[1].0, &ctx.green_tile);
        }

        #[test]
        fn test_independent_strategy_allows_duplicate_neighbours() {
            let ctx = setup();

            let red_image = red_image(&ctx);
            let analysis = analyse_tiles(&ctx, vec![&ctx.red_tile1, &ctx.red_tile2]);
            let strategy =
                IndependentStrategy::new(&analysis, &ctx.analysis_options, ctx.cell_size);

            let result = strategy.choose(&red_image);

            assert_eq!(result.len(), 3);
            let result = sort_by_position(&result);
            assert_eq!(result[0].0, result[1].0);
            assert_eq!(result[0].0, result[2].0);
            assert_eq!(result[1].0, result[2].0);
        }
    }

    mod holistic_strategy {
        use crate::strategy::strategy_tests::*;

        #[test]
        fn test_holistic_strategy_picks_best_match() {
            let ctx = setup();

            let blue_image = blue_image(&ctx);
            let analysis = analyse_tiles(&ctx, vec![&ctx.blue_tile, &ctx.green_tile]);
            let strategy =
                HolisticStrategy::new(&analysis, &ctx.analysis_options, ctx.cell_size, |_| 10);

            let result = strategy.choose(&blue_image);

            assert_eq!(result.len(), 1);
            assert_eq!(result[0].0, &ctx.blue_tile);
        }

        #[test]
        fn test_holistic_strategy_picks_best_match_for_each_cell() {
            let ctx = setup();

            let blue_green_image = blue_green_image(&ctx);
            let analysis = analyse_tiles(&ctx, vec![&ctx.blue_tile, &ctx.green_tile]);
            let strategy =
                HolisticStrategy::new(&analysis, &ctx.analysis_options, ctx.cell_size, |_| 10);

            let result = strategy.choose(&blue_green_image);

            assert_eq!(result.len(), 2);
            let result = sort_by_position(&result);
            assert_eq!(result[0].0, &ctx.blue_tile);
            assert_eq!(result[1].0, &ctx.green_tile);
        }

        #[test]
        fn test_holistic_strategy_avoids_duplicate_neighbours() {
            let ctx = setup();

            let red_image = red_image(&ctx);
            let analysis = analyse_tiles(&ctx, vec![&ctx.red_tile1, &ctx.red_tile2]);
            let strategy =
                HolisticStrategy::new(&analysis, &ctx.analysis_options, ctx.cell_size, |_| 10);

            let result = strategy.choose(&red_image);

            assert_eq!(result.len(), 3);
            let result = sort_by_position(&result);
            assert_ne!(result[0].0, result[1].0);
        }

        #[test]
        fn test_holistic_strategy_avoids_multiple_duplicate_neighbours() {
            let ctx = setup();

            let red_image = red_image(&ctx);
            let analysis = analyse_tiles(&ctx, vec![&ctx.red_tile1, &ctx.red_tile2, &ctx.red_tile3]);
            let strategy =
                HolisticStrategy::new(&analysis, &ctx.analysis_options, ctx.cell_size, |_| 10);

            let result = strategy.choose(&red_image);

            assert_eq!(result.len(), 3);
            let result = sort_by_position(&result);
            assert_ne!(result[0].0, result[1].0);
            assert_ne!(result[0].0, result[2].0);
            assert_ne!(result[1].0, result[2].0);
        }
    }
}
