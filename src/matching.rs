use std::collections::HashMap;

use image::{imageops, GenericImageView, RgbaImage};
use num::pow;

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
        let target_info = analyse_cell(img, r, self.options);
        let best_tile = *self
            .analysis
            .iter()
            .min_by_key(|(_, info)| info.diff(&target_info).iter().sum::<i32>())
            .unwrap()
            .0;
        (best_tile, PixelRegion::from(r))
    }

    // Holistic tile selection

    #[allow(dead_code)]
    pub fn choose2(
        &self,
        target: &RgbaImage,
        cell_size: &Dimensions,
    ) -> Vec<TileLocation<T, PixelRegion>> {
        let _cells_info: Vec<(&Rectangle, ImageInfo)> = grid(target, cell_size)
            .iter()
            .map(|t| (t, analyse_cell(target, t, self.options)))
            .collect();
        todo!();
    }
}

#[allow(dead_code)]
#[derive(Hash, PartialEq, Debug)]
struct CellCoords {
    x: i32,
    y: i32,
}

trait Surrounded<T> {
    /// Find the surrounding items
    fn surrounding(&self, distance: i32) -> Vec<T>;
}

impl Surrounded<CellCoords> for CellCoords {
    fn surrounding(&self, distance: i32) -> Vec<CellCoords> {
        let (xmin, xmax) = (self.x - distance, self.x + distance);
        let (ymin, ymax) = (self.y - distance, self.y + distance);

        let xs = xmin..=xmax;
        let ys = ymin..=ymax;

        let d2 = pow(distance, 2);

        itertools::iproduct!(xs, ys)
            .filter_map(|(x, y)| {
                let x_dist = x - self.x;
                let y_dist = y - self.y;
                let d = pow(x_dist, 2) + pow(y_dist, 2);
                println!("{},{} => {}", x, y, d);
                if d <= d2 {
                    Some(CellCoords { x, y })
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod surrounding_test {
    use itertools::Itertools;

    use super::CellCoords;
    use super::Surrounded;

    #[test]
    fn test_finds_self() {
        let target = CellCoords { x: 5, y: 5 };

        let results = target.surrounding(0);

        assert_eq!(results.len(), 1);

        assert!(results.contains(&CellCoords { x: 5, y: 5 }));
    }

    #[test]
    fn test_finds_immediate_neighbours() {
        let target = CellCoords { x: 2, y: 2 };

        let results = target.surrounding(1);

        assert_eq!(results.len(), 5);
        assert_eq!(map(&results, (0,0), (3,3)), concat!(
            "....",
            "..x.",
            ".xxx",
            "..x.",
        ));
    }

    #[test]
    fn test_finds_neighbours_within_radius() {
        let target = CellCoords { x: 2, y: 2 };

        let results = target.surrounding(2);

        assert_eq!(results.len(), 13);
        assert_eq!(map(&results, (0,0), (4,4)), concat!(
            "..x..",
            ".xxx.",
            "xxxxx",
            ".xxx.",
            "..x..",
        ));
    }

    #[test]
    fn test_goes_negative() {
        let target = CellCoords { x: 0, y: 0 };

        let results = target.surrounding(1);

        assert_eq!(results.len(), 5);
        assert_eq!(map(&results, (-1,-1), (1,1)), concat!(
            ".x.",
            "xxx",
            ".x.",
        ));
    }

    fn map(results: &Vec<CellCoords>, min: (i32, i32), max: (i32,i32)) -> String {
        (min.1..=max.1)
            .map(|y|
                (min.0..=max.0)
                .map(|x| if results.contains(&CellCoords { x, y }) {"x"} else {"."})
                .join("")
            )
            .join("")
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
    analyse(&target.to_image(), options) }

#[cfg(test)]
mod test {
    // use super::*;

    // #[test]
    // fn test_foo() {
    //     todo!();
    // }
}
