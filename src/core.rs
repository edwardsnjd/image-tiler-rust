/// Alias for width and height
pub type Dimensions = (u32, u32);

/// Alias for pixel position
pub type Point = (i64, i64);

/// Convenience type alias for a tile and where to draw it
pub type TileLocation<'a, T> = (&'a T, Point, Dimensions);

/// Extension trait for TileLocation (since it's a built in type)
pub trait TileLocationExtensions<T> {
    /// Scale the size and position of the tile location
    fn scale(&self, ratio: u32) -> TileLocation<T>;
}

impl<T> TileLocationExtensions<T> for TileLocation<'_, T> {
    fn scale(&self, ratio: u32) -> TileLocation<T> {
        let (p, position, size) = self;
        (
            p,
            position.map(|v| v * (ratio as i64)),
            size.map(|v| v * ratio),
        )
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
    T: std::ops::Mul<Output = T> + Copy
{
    fn map<F, U>(&self, f: F) -> (U, U)
    where
        F: Fn(&T) -> U
    {
        (f(&self.0), f(&self.1))
    }

    fn scale(&self, ratio: T) -> (T, T) {
        self.map(|v| *v * ratio)
    }
}
