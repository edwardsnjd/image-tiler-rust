/// Alias for width and height
pub type Dimensions = (u32, u32);

/// Alias for pixel position
pub type Point = (i64, i64);

/// Convenience type alias for a tile and where to draw it
pub type TileLocation<'a, T> = (&'a T, Point, Dimensions);
