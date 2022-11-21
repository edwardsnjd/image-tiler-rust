use core::fmt::Debug;

use image::{imageops, Pixel, RgbaImage};

#[allow(dead_code)]
const SAMPLE_SIZE: u8 = 8;

#[allow(dead_code)]
pub fn analyse(img: &RgbaImage, options: &AnalysisOptions) -> ImageInfo {
    let size = options.sample_size as u32;
    let (width, height) = img.dimensions();

    // Resize image as a simple way to get pixel data
    let tiny_version = imageops::thumbnail(img, size, size);

    let colors = tiny_version
        .pixels()
        .map(|p| {
            let vals = p.channels();
            ColorInfo {
                red: vals[0].to_owned(),
                blue: vals[1].to_owned(),
                green: vals[2].to_owned(),
            }
        })
        .collect();

    ImageInfo {
        width,
        height,
        colors,
    }
}

pub struct AnalysisOptions {
    pub sample_size: u8,
}

#[allow(dead_code)]
pub fn options(sample_size: Option<u8>) -> AnalysisOptions {
    AnalysisOptions {
        sample_size: sample_size.unwrap_or(SAMPLE_SIZE),
    }
}

/// Data describing the image, suitable for comparison between images.
#[derive(Debug, PartialEq, Eq)]
pub struct ImageInfo {
    width: u32,
    height: u32,
    colors: Vec<ColorInfo>,
}

/// Data describing the color of a pixel.
#[derive(PartialEq, Eq)]
pub struct ColorInfo {
    red: u8,
    blue: u8,
    green: u8,
}

impl Debug for ColorInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({},{},{})", self.red, self.blue, self.green)
    }
}

impl ColorInfo {
    #[allow(dead_code)]
    fn new(red: u8, green: u8, blue: u8) -> ColorInfo {
        Self { red, green, blue }
    }

    #[allow(dead_code)]
    fn diff(&self, other: &ColorInfo) -> i32 {
        num::abs(self.red as i32 - other.red as i32)
            + num::abs(self.green as i32 - other.green as i32)
            + num::abs(self.blue as i32 - other.blue as i32)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct TestContext {
        black: ColorInfo,
        red: ColorInfo,
        green: ColorInfo,
        blue: ColorInfo,
        grey: ColorInfo,
        white: ColorInfo,
    }

    fn setup() -> TestContext {
        TestContext {
            black: ColorInfo::new(0, 0, 0),
            red: ColorInfo::new(255, 0, 0),
            blue: ColorInfo::new(0, 255, 0),
            green: ColorInfo::new(0, 0, 255),
            grey: ColorInfo::new(127, 127, 127),
            white: ColorInfo::new(255, 255, 255),
        }
    }

    #[test]
    fn test_returns_expected_analysis() {
        let ctx = setup();
        let size = 100;
        let img = RgbaImage::new(size, size);
        let opts = options(Some(1));

        let result = analyse(&img, &opts);

        assert_eq!(
            result,
            ImageInfo {
                width: size,
                height: size,
                colors: vec![ctx.black],
            }
        );
    }

    #[test]
    fn test_returns_absolute_color_difference() {
        let ctx = setup();
        assert_eq!(ctx.black.diff(&ctx.black), 0);
        assert_eq!(ctx.red.diff(&ctx.red), 0);
        assert_eq!(ctx.green.diff(&ctx.green), 0);
        assert_eq!(ctx.blue.diff(&ctx.blue), 0);
        assert_eq!(ctx.grey.diff(&ctx.grey), 0);
        assert_eq!(ctx.white.diff(&ctx.white), 0);

        assert_eq!(ctx.black.diff(&ctx.red), 255 + 0 + 0);
        assert_eq!(ctx.black.diff(&ctx.green), 0 + 255 + 0);
        assert_eq!(ctx.black.diff(&ctx.blue), 0 + 0 + 255);
        assert_eq!(ctx.black.diff(&ctx.grey), 127 + 127 + 127);
        assert_eq!(ctx.black.diff(&ctx.white), 255 + 255 + 255);
    }
}
