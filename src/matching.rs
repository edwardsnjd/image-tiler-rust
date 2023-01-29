use std::collections::HashMap;

use image::{imageops, GenericImageView, RgbaImage};
use num::pow;

use crate::analysis::{analyse, AnalysisOptions, ImageInfo};
use crate::core::{Dimensions, PixelRegion, Rectangle, TileLocation};

pub struct MatchingTileStrategy<'a, T> {
    options: &'a AnalysisOptions,
    analysis: &'a HashMap<&'a T, ImageInfo>,
}

impl<T: std::hash::Hash + std::cmp::Eq + std::fmt::Debug> MatchingTileStrategy<'_, T> {
    pub fn new<'a>(
        analysis: &'a HashMap<&T, ImageInfo>,
        options: &'a AnalysisOptions,
    ) -> MatchingTileStrategy<'a, T> {
        MatchingTileStrategy { options, analysis }
    }

    // Independent tile selection

    #[allow(dead_code)]
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

    #[allow(dead_code)]
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
        let cells_ranked: Vec<(CellCoords, HashMap<&T, i32>)> = grid2(target, cell_size)
            .into_iter()
            .map(|c| (c, c.to_rect(cell_size)))
            .map(|(c, t)| (c, analyse_cell(target, &t, self.options)))
            .map(|(c, i)| (c, self.rank_library(&i)))
            .collect();

        eprintln!("Writing rankings");
        for (c, ranked) in cells_ranked.iter().take(3) {
            eprintln!("{:#?}", c);
            for (p, s) in ranked.iter().take(3) {
                eprintln!("{:#?} = {:?}", p, s);
            }
        }

        // Calculate penalties
        let mut penalties: HashMap<CellCoords, HashMap<&T, i32>> = HashMap::new();
        for (c, ranked) in cells_ranked.iter() {
            // Penalise score for best in nearby cells to according to proximity
            let best = self.pick_best(ranked);
            for sc in c.surrounding(7) {
                let penalty = match sc.sqr_distance(c) {
                    0 => 0,
                    1 => 20 * 20 * 200,
                    2..=6 => 20 * 20 * 100,
                    5..=9 => 20 * 20 * 50,
                    _ => 0,
                };

                penalties
                    .entry(sc)
                    .and_modify(|target_penalties| {
                        target_penalties
                            .entry(best)
                            .and_modify(|total| *total += penalty)
                            .or_insert(penalty);
                    })
                    .or_insert_with(|| HashMap::from([(best, penalty)]));
            }
        }

        // Recalculate rankings including penalties
        let cells_adjusted: Vec<(CellCoords, HashMap<&T, i32>)> = cells_ranked
            .into_iter()
            .map(|(c, ranked)| {
                let foo = ranked
                    .iter()
                    .map(|(&t, score)| {
                        let bar = penalties.get(&c).map_or(0, |h| *h.get(t).unwrap_or(&0));
                        (t, score - bar)
                    })
                    .collect();
                (c, foo)
            })
            .collect();

        cells_adjusted
            .iter()
            .map(|(c, ranked)| (self.pick_best(ranked), c.to_region(cell_size)))
            .collect()
    }

    fn rank_library(&self, info: &ImageInfo) -> HashMap<&T, i32> {
        self.analysis
            .iter()
            .map(|(&t, i)| (t, i.diff(info).iter().sum::<i32>()))
            .collect()
    }

    fn pick_best<'a>(&self, cells_ranked: &HashMap<&'a T, i32>) -> &'a T {
        cells_ranked
            .iter()
            .min_by(|a, b| a.1.cmp(b.1))
            .unwrap()
            .0
    }
}

#[allow(dead_code)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
struct CellCoords {
    x: i32,
    y: i32,
}

impl CellCoords {
    #[allow(dead_code)]
    fn abs_distance(&self, other: &Self) -> (i32, i32) {
        let x_dist = other.x - self.x;
        let y_dist = other.y - self.y;
        (x_dist, y_dist)
    }

    #[allow(dead_code)]
    fn sqr_distance(&self, other: &Self) -> i32 {
        let (x_dist, y_dist) = self.abs_distance(other);
        pow(x_dist, 2) + pow(y_dist, 2)
    }

    fn to_rect(self, cell_size: &Dimensions) -> Rectangle {
        let (cw, ch) = cell_size;
        let (x, y) = (self.x as u32 * *cw, self.y as u32 * *ch);
        Rectangle::new(x, y, *cw, *ch)
    }

    fn to_region(self, cell_size: &Dimensions) -> PixelRegion {
        let (cw, ch) = cell_size;
        let (x, y) = (self.x as i64 * *cw as i64, self.y as i64 * *ch as i64);
        PixelRegion::new(x, y, *cw, *ch)
    }
}

#[cfg(test)]
mod cellcoords_test {
    use super::CellCoords;

    #[test]
    fn test_distance_is_zero_for_same() {
        let target = CellCoords { x: 5, y: 5 };

        let d = target.abs_distance(&target);
        let d2 = target.sqr_distance(&target);

        assert_eq!(d, (0, 0));
        assert_eq!(d2, 0);
    }

    #[test]
    fn test_distance_is_1_for_bordering() {
        let target = CellCoords { x: 5, y: 5 };

        let borders = vec![
            CellCoords { x: 4, y: 5 },
            CellCoords { x: 6, y: 5 },
            CellCoords { x: 5, y: 4 },
            CellCoords { x: 5, y: 6 },
        ];

        let ds: Vec<(i32, i32)> = borders
            .iter()
            .map(|border| target.abs_distance(border))
            .collect();
        let d2s: Vec<i32> = borders
            .iter()
            .map(|border| target.sqr_distance(border))
            .collect();

        assert_eq!(ds, vec!((-1, 0), (1, 0), (0, -1), (0, 1)));
        assert_eq!(d2s, vec!(1, 1, 1, 1));
    }

    #[test]
    fn test_distance_is_more_further_out() {
        let target = CellCoords { x: 5, y: 5 };

        let borders = vec![
            CellCoords { x: 4, y: 4 },
            CellCoords { x: 6, y: 6 },
            CellCoords { x: 4, y: 4 },
            CellCoords { x: 6, y: 6 },
        ];

        let ds: Vec<(i32, i32)> = borders
            .iter()
            .map(|border| target.abs_distance(border))
            .collect();
        let d2s: Vec<i32> = borders
            .iter()
            .map(|border| target.sqr_distance(border))
            .collect();

        assert_eq!(ds, vec!((-1, -1), (1, 1), (-1, -1), (1, 1)));
        assert_eq!(d2s, vec!(2, 2, 2, 2));
    }
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
            .map(|(x, y)| CellCoords { x, y })
            .filter(|candidate| self.sqr_distance(candidate) <= d2)
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
        let results = CellCoords { x: 5, y: 5 }.surrounding(0);

        assert_eq!(results.len(), 1);
        assert!(results.contains(&CellCoords { x: 5, y: 5 }));
    }

    #[test]
    #[rustfmt::skip]
    fn test_finds_immediate_neighbours() {
        let results = CellCoords { x: 2, y: 2 }.surrounding(1);

        assert_eq!(results.len(), 5);
        assert_eq!(map(&results, (0, 0), (3, 3)), concat!(
            "....",
            "..x.",
            ".xxx",
            "..x.",
        ));
    }

    #[test]
    #[rustfmt::skip]
    fn test_finds_neighbours_within_radius() {
        let results = CellCoords { x: 2, y: 2 }.surrounding(2);

        assert_eq!(results.len(), 13);
        assert_eq!(map(&results, (0, 0), (4, 4)), concat!(
            "..x..",
            ".xxx.",
            "xxxxx",
            ".xxx.",
            "..x..",
        ));
    }

    #[test]
    #[rustfmt::skip]
    fn test_goes_negative() {
        let results = CellCoords { x: 0, y: 0 }.surrounding(1);

        assert_eq!(results.len(), 5);
        assert_eq!(map(&results, (-1, -1), (1, 1)), concat!(
            ".x.",
            "xxx",
            ".x.",
        ));
    }

    // Convert results to a visual map over given bounds
    fn map(results: &[CellCoords], min: (i32, i32), max: (i32, i32)) -> String {
        (min.1..=max.1)
            .map(|y| {
                (min.0..=max.0)
                    .map(|x| {
                        if results.contains(&CellCoords { x, y }) {
                            "x"
                        } else {
                            "."
                        }
                    })
                    .join("")
            })
            .join("")
    }
}

#[allow(dead_code)]
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

fn grid2<I>(target: &I, cell_size: &Dimensions) -> Vec<CellCoords>
where
    I: GenericImageView,
{
    let (tw, th) = target.dimensions();
    let (cw, ch) = cell_size;

    let (across, down) = (1 + tw / cw, 1 + th / ch);

    let xs = 0..across as i32;
    let ys = 0..down as i32;

    itertools::iproduct!(xs, ys)
        .map(|(x, y)| CellCoords { x, y })
        .collect()
}

fn analyse_cell(img: &RgbaImage, r: &Rectangle, options: &AnalysisOptions) -> ImageInfo {
    let target = imageops::crop_imm(img, r.x, r.y, r.width, r.height);
    analyse(&target.to_image(), options)
}
