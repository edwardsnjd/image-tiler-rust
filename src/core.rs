/// Alias for width and height
pub type Dimensions = (u32, u32);

/// Convenience type alias for a tile and where to draw it
pub type TileLocation<'a, T, U> = (&'a T, U);

#[derive(Eq, PartialEq, Debug, Hash)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }
}

/// The position of a tile expressed in terms of pixel coords.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct PixelRegion {
    pub x: i64,
    pub y: i64,
    pub width: u32,
    pub height: u32,
}

impl PixelRegion {
    pub fn new(x: i64, y: i64, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    pub fn from(r: &Rectangle) -> Self {
        Self::new(r.x.into(), r.y.into(), r.width, r.height)
    }

    pub fn scale(&self, ratio: u32) -> Self {
        Self::new(
            self.x * (ratio as i64),
            self.y * (ratio as i64),
            self.width * ratio,
            self.height * ratio,
        )
    }
}

/// Extension trait for TileLocation (since it's a built in type)
pub trait TileLocationExtensions<T, U> {
    /// Scale the size and position of the tile location
    fn scale(&self, ratio: u32) -> TileLocation<T, U>;
}

impl<T> TileLocationExtensions<T, PixelRegion> for TileLocation<'_, T, PixelRegion> {
    fn scale(&self, ratio: u32) -> TileLocation<T, PixelRegion> {
        let (p, region) = self;
        (p, region.scale(ratio))
    }
}

/// Extension trait for homogenous tuple pairs (since it's a built in type)
pub trait TupleExtensions<T> {
    /// Map the values inside the tuple
    fn map<F, U>(&self, f: F) -> (U, U)
    where
        F: Fn(&T) -> U;

    /// Scale the numeric values inside the tuple
    fn scale(&self, ratio: T) -> (T, T);
}

impl<T> TupleExtensions<T> for (T, T)
where
    T: std::ops::Mul<Output = T> + Copy,
{
    fn map<F, U>(&self, f: F) -> (U, U)
    where
        F: Fn(&T) -> U,
    {
        (f(&self.0), f(&self.1))
    }

    fn scale(&self, ratio: T) -> (T, T) {
        self.map(|v| *v * ratio)
    }
}

/// A view into a grid of items, represented as a linear slice.
#[allow(dead_code)]
pub struct GridView<'a, T> {
    items: &'a [T],
    dimensions: Dimensions,
    region: Rectangle,
}

#[allow(dead_code)]
impl<'a, T> GridView<'a, T> {
    /// Create a view that covers the entire input.
    fn all(items: &'a [T], dimensions: Dimensions) -> Self {
        let (width, height) = dimensions;
        let region = Rectangle::new(0, 0, width, height);

        Self::new(items, dimensions, region)
    }

    /// Create a view over a specific region of the input.
    fn new(items: &'a [T], dimensions: Dimensions, region: Rectangle) -> Self {
        let (width, height) = dimensions;

        let count = items.len();
        if count != (width * height) as usize {
            panic!("Dimensions, {dimensions:?}, do not work with {count} items");
        }

        let (max_x, max_y) = (region.x + region.width - 1, region.y + region.height - 1);
        if max_x >= width || max_y >= height {
            panic!("Region, {region:?}, is out of bounds for dimensions {dimensions:?}");
        }

        Self {
            items,
            dimensions,
            region,
        }
    }

    /// Get the item at the specified offset within the region.
    fn get(&self, dx: u32, dy: u32) -> &T {
        let Rectangle {
            x: rx,
            y: ry,
            width: rwidth,
            height: rheight,
        } = self.region;
        if dx >= rwidth || dy >= rheight {
            let size = (rwidth, rheight);
            panic!("Point, ({dx}, {dy}), out of bounds for region size {size:?}");
        }

        let (width, _) = self.dimensions;
        let (x, y) = (rx + dx, ry + dy);
        let index = x + (y * width);

        &self.items[index as usize]
    }

    /// Create a new view that is a subset of the current view.
    fn subset(&self, target: Rectangle) -> GridView<T> {
        GridView::new(
            self.items,
            self.dimensions,
            Rectangle {
                x: self.region.x + target.x,
                y: self.region.y + target.y,
                width: target.width,
                height: target.height,
            },
        )
    }
}

#[cfg(test)]
mod grid_view_tests {
    use super::GridView;
    use crate::core::Rectangle;

    // GridView#all

    #[test]
    #[should_panic]
    fn grid_rejects_invalid_dimensions() {
        let vals = vec![0, 1, 2, 3, 4];
        let dims = (2, 3);

        GridView::all(&vals, dims);
    }

    #[test]
    fn grid_accepts_valid_dimensions() {
        let vals = vec![0, 1, 2, 3, 4, 5];
        let dims = (2, 3);

        GridView::all(&vals, dims);
    }

    // GridView#new

    #[test]
    #[should_panic]
    fn grid_rejects_invalid_region() {
        let vals = vec![0, 1, 2, 3, 4, 5];
        let dims = (2, 3);

        GridView::new(&vals, dims, Rectangle::new(1, 1, 2, 3));
    }

    // GridView#get

    #[test]
    fn grid_get_finds_correct_items() {
        let vals = vec![0, 1, 2, 3, 4, 5];
        let dims = (2, 3);
        let grid = GridView::new(&vals, dims, Rectangle::new(0, 0, 2, 3));

        assert_eq!(grid.get(0, 0), &0);
        assert_eq!(grid.get(1, 0), &1);
        assert_eq!(grid.get(0, 1), &2);
        assert_eq!(grid.get(1, 1), &3);
        assert_eq!(grid.get(0, 2), &4);
        assert_eq!(grid.get(1, 2), &5);
    }

    #[test]
    #[should_panic]
    fn grid_get_rejects_invalid_index() {
        let vals = vec![0, 1, 2, 3, 4, 5];
        let dims = (2, 3);

        let grid = GridView::new(&vals, dims, Rectangle::new(0, 0, 2, 3));

        grid.get(2, 0);
    }

    // GridView#subset

    #[test]
    fn grid_subset_finds_correct_items() {
        let vals = vec![0, 1, 2, 3, 4, 5];
        let dims = (2, 3);
        let grid = GridView::new(&vals, dims, Rectangle::new(0, 0, 2, 3));

        let result = grid.subset(Rectangle::new(1, 1, 1, 1));

        assert_eq!(result.get(0, 0), &3);
    }

    #[test]
    #[should_panic]
    fn grid_subset_rejects_invalid_target() {
        let vals = vec![0, 1, 2, 3, 4, 5];
        let dims = (2, 3);
        let grid = GridView::new(&vals, dims, Rectangle::new(0, 0, 2, 3));

        grid.subset(Rectangle::new(2, 2, 2, 2));
    }
}
