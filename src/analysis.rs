use core::fmt::Debug;

use image::{imageops, Pixel, RgbaImage};

const SAMPLE_SIZE: u8 = 8;

pub fn analyse(img: &RgbaImage, options: &AnalysisOptions) -> ImageInfo {
    let size = options.sample_size as u32;
    let (width, height) = img.dimensions();

    // Resize image as a simple way to get pixel data
    let tiny_version = imageops::thumbnail(img, size, size);

    let colors = tiny_version
        .pixels()
        .map(|p| {
            let vals = p.channels();
            ColorInfo::new(
                vals[0].to_owned(),
                vals[1].to_owned(),
                vals[2].to_owned(),
            )
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

impl AnalysisOptions {
    pub fn new(sample_size: Option<u8>) -> AnalysisOptions {
        Self {
            sample_size: sample_size.unwrap_or(SAMPLE_SIZE),
        }
    }
}

/// Data describing the image, suitable for comparison between images.
#[derive(Debug, PartialEq, Eq)]
pub struct ImageInfo {
    width: u32,
    height: u32,
    colors: Vec<ColorInfo>,
}

impl ImageInfo {
    pub fn diff(&self, other: &ImageInfo) -> Vec<i32> {
        let (this, that) = (&self.colors, &other.colors);

        assert!(this.len() == that.len());

        let pairs: Vec<(&ColorInfo, &ColorInfo)> = this.iter().zip(that.iter()).collect();

        pairs.iter().map(|(a, b)| a.abs_diff(b)).collect()
    }
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
        write!(f, "ColorInfo({},{},{})", self.red, self.blue, self.green)
    }
}

impl ColorInfo {
    fn new(red: u8, green: u8, blue: u8) -> ColorInfo {
        Self { red, green, blue }
    }

    fn abs_diff(&self, other: &ColorInfo) -> i32 {
        let df = |a,b| num::abs(a - b);
        df(self.red as i32, other.red as i32)
            + df(self.green as i32, other.green as i32)
            + df(self.blue as i32, other.blue as i32)
    }

    #[allow(dead_code)]
    fn sqr_diff(&self, other: &ColorInfo) -> i32 {
        let df = |a,b| num::pow(a - b, 2);
        df(self.red as i32, other.red as i32)
            + df(self.green as i32, other.green as i32)
            + df(self.blue as i32, other.blue as i32)
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
    fn test_returns_expected_image_analysis() {
        let ctx = setup();
        let size = 100;
        let img = RgbaImage::new(size, size);
        let opts = AnalysisOptions::new(Some(1));

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
    fn test_returns_absolute_image_color_difference() {
        let ctx = setup();
        assert_eq!(ctx.black.abs_diff(&ctx.black), 0);
        assert_eq!(ctx.red.abs_diff(&ctx.red), 0);
        assert_eq!(ctx.green.abs_diff(&ctx.green), 0);
        assert_eq!(ctx.blue.abs_diff(&ctx.blue), 0);
        assert_eq!(ctx.grey.abs_diff(&ctx.grey), 0);
        assert_eq!(ctx.white.abs_diff(&ctx.white), 0);

        assert_eq!(ctx.black.abs_diff(&ctx.red), 255 + 0 + 0);
        assert_eq!(ctx.black.abs_diff(&ctx.green), 0 + 255 + 0);
        assert_eq!(ctx.black.abs_diff(&ctx.blue), 0 + 0 + 255);
        assert_eq!(ctx.black.abs_diff(&ctx.grey), 127 + 127 + 127);
        assert_eq!(ctx.black.abs_diff(&ctx.white), 255 + 255 + 255);
    }

    #[test]
    fn test_returns_squared_image_color_difference() {
        let ctx = setup();
        assert_eq!(ctx.black.sqr_diff(&ctx.black), 0);
        assert_eq!(ctx.red.sqr_diff(&ctx.red), 0);
        assert_eq!(ctx.green.sqr_diff(&ctx.green), 0);
        assert_eq!(ctx.blue.sqr_diff(&ctx.blue), 0);
        assert_eq!(ctx.grey.sqr_diff(&ctx.grey), 0);
        assert_eq!(ctx.white.sqr_diff(&ctx.white), 0);

        assert_eq!(ctx.black.sqr_diff(&ctx.red), 255*255 + 0 + 0);
        assert_eq!(ctx.black.sqr_diff(&ctx.green), 0 + 255*255 + 0);
        assert_eq!(ctx.black.sqr_diff(&ctx.blue), 0 + 0 + 255*255);
        assert_eq!(ctx.black.sqr_diff(&ctx.grey), 127*127 + 127*127 + 127*127);
        assert_eq!(ctx.black.sqr_diff(&ctx.white), 255*255 + 255*255 + 255*255);
    }

    #[test]
    fn test_returns_zero_diffs_for_identical_images() {
        let size = 100;
        let img1 = RgbaImage::new(size, size);
        let img2 = RgbaImage::new(size, size);

        let opts = AnalysisOptions::new(Some(2));

        let result1 = analyse(&img1, &opts);
        let result2 = analyse(&img2, &opts);

        let diffs = result1.diff(&result2);

        assert_eq!(diffs, vec![0, 0, 0, 0]);
    }

    #[test]
    fn test_returns_diff_of_each_sample() {
        let size = 100;
        let img1 = RgbaImage::new(size, size);
        let img2 = RgbaImage::from_pixel(size, size, image::Rgba([0, 0, 255, 0]));

        let opts = AnalysisOptions::new(Some(2));

        let result1 = analyse(&img1, &opts);
        let result2 = analyse(&img2, &opts);

        let diffs = result1.diff(&result2);

        assert_eq!(diffs, vec![255, 255, 255, 255]);
    }
}
