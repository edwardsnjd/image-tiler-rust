/// Alias for width and height
pub type Dimensions = (u32, u32);

/// Convenience type alias for a tile and where to draw it
pub type TileLocation<'a, T> = (&'a T, PixelRegion);

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
pub trait TileLocationExtensions<T> {
    /// Scale the size and position of the tile location
    fn scale(&self, ratio: u32) -> TileLocation<T>;
}

impl<T> TileLocationExtensions<T> for TileLocation<'_, T> {
    fn scale(&self, ratio: u32) -> TileLocation<T> {
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
