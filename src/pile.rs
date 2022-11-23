use image::RgbaImage;
use rand::thread_rng;
use rand::Rng;

use crate::core::Dimensions;
use crate::core::TileLocation;
use crate::core::TileStrategy;

/// Minimum number of tiles to draw (repeat tiles if fewer than this)
const MIN_TILES: usize = 128;

// Random pile

pub struct RandomPileStrategy<'a> {
    tiles: &'a [RgbaImage],
    min_tiles: usize,
}

pub fn random_pile_strategy(
    tiles: &[RgbaImage],
    min_tiles: Option<usize>,
) -> RandomPileStrategy {
    let min_tiles = min_tiles.unwrap_or(MIN_TILES);
    RandomPileStrategy { tiles, min_tiles }
}

impl TileStrategy for RandomPileStrategy<'_> {
    fn choose(&self, target: &RgbaImage) -> Vec<TileLocation<RgbaImage>> {
        random_pile(self.tiles, self.min_tiles, target.dimensions())
    }
}

/// Trait capturing any dimensioned structure
trait Dimensioned {
    fn dimensions(&self) -> Dimensions;
}

/// Explicitly treat images as dimensioned
impl Dimensioned for RgbaImage {
    fn dimensions(&self) -> Dimensions {
        self.dimensions()
    }
}

/// Place tiles in a random pile
fn random_pile<T>(tiles: &[T], min_tiles: usize, size: Dimensions) -> Vec<TileLocation<T>>
where
    T: Dimensioned,
{
    let tiles_to_place = min_tiles.max(tiles.len());
    let (width, height) = size;

    let mut rng = thread_rng();
    let mut generate_random_coords = |(w, h)| {
        (
            rng.gen_range(-(w as i64)..width as i64),
            rng.gen_range(-(h as i64)..height as i64),
        )
    };

    tiles
        .iter()
        .cycle()
        .take(tiles_to_place)
        .map(|tile| {
            let size = tile.dimensions();
            let (x, y) = generate_random_coords(size);
            (tile, x, y)
        })
    .collect()
}

#[cfg(test)]
mod test {
    use super::*;

    struct FakeImage {
        width: u32,
        height: u32,
    }

    fn fake_image(width: u32, height: u32) -> FakeImage {
        FakeImage { width, height }
    }

    impl Dimensioned for FakeImage {
        fn dimensions(&self) -> Dimensions {
            (self.width, self.height)
        }
    }

    #[test]
    fn test_returns_zero_tiles_for_no_input() {
        let tiles: Vec<FakeImage> = vec![];
        let actual = random_pile(&tiles, 2, (100, 200));
        assert_eq!(actual.len(), 0);
    }

    #[test]
    fn test_returns_minimum_number_even_if_insufficient_tiles() {
        let tiles = vec![fake_image(10, 10)];
        let actual = random_pile(&tiles, 7, (100, 200));
        assert_eq!(actual.len(), 7);
    }

    #[test]
    fn test_all_coords_in_bounds() {
        let (width, height): Dimensions = (100, 200);
        let tile_size: u32 = 10;
        let tiles = vec![fake_image(tile_size, tile_size)];

        let actual = random_pile(&tiles, 7, (width, height));

        let xcoords: Vec<i64> = actual.iter().map(|loc| loc.1).collect();
        let ycoords: Vec<i64> = actual.iter().map(|loc| loc.2).collect();

        let all_x_valid = xcoords
            .iter()
            .all(|x| -(tile_size as i64) <= *x && *x < width as i64);
        let all_y_valid = ycoords
            .iter()
            .all(|y| -(tile_size as i64) <= *y && *y < height as i64);

        assert_eq!(all_x_valid, true, "{:?}", xcoords);
        assert_eq!(all_y_valid, true, "{:?}", ycoords);
    }
}
