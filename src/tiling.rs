/// Choose the area to use as a tile from an image of the given dimensions.
pub fn choose_tile_area(width: u32, height: u32) -> Rectangle {
    let (x, y, s) = if width < height {
        (0, (height - width) / 2, width)
    } else {
        ((width - height) / 2, 0, height)
    };

    Rectangle::new(x, y, s, s)
}

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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_chooses_central_square_for_portrait_tile() {
        assert_eq!(choose_tile_area(10, 20), Rectangle::new(0, 5, 10, 10));
    }

    #[test]
    fn test_chooses_central_square_for_landscape_tile() {
        assert_eq!(choose_tile_area(20, 10), Rectangle::new(5, 0, 10, 10));
    }
}
