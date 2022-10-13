use sciimg::{decompanding, enums, rgbimage};

const M20_ZCAM_ECM_GRAY: &str =
    "tests/testdata/ZL0_0038_0670307360_057ECM_N0031392ZCAM08007_1100LUJ.png";
const M20_ZCAM_ECM_RGB: &str =
    "tests/testdata/ZL0_0053_0671642352_402ECM_N0032046ZCAM05025_110085J01.png";

#[test]
fn test_grayscale_check() {
    let img_gray = rgbimage::RgbImage::open(&String::from(M20_ZCAM_ECM_GRAY)).unwrap();
    assert_eq!(img_gray.is_grayscale(), true);

    let img_rgb = rgbimage::RgbImage::open(&String::from(M20_ZCAM_ECM_RGB)).unwrap();
    assert_eq!(img_rgb.is_grayscale(), false);
}

#[test]
fn test_image_size() {
    let img_rgb = rgbimage::RgbImage::open(&String::from(M20_ZCAM_ECM_RGB)).unwrap();
    assert_eq!(img_rgb.width, 1648);
    assert_eq!(img_rgb.height, 1200);
}

#[test]
fn test_image_mode() {
    let mut img_rgb = rgbimage::RgbImage::open(&String::from(M20_ZCAM_ECM_RGB)).unwrap();
    assert_eq!(img_rgb.get_mode(), enums::ImageMode::U8BIT);
    img_rgb.decompand(&decompanding::ILT);
    assert_eq!(img_rgb.get_mode(), enums::ImageMode::U12BIT);
    img_rgb.normalize_to_16bit_with_max(2033.0);
    assert_eq!(img_rgb.get_mode(), enums::ImageMode::U16BIT);
    img_rgb.normalize_to_12bit_with_max(2033.0, 65535.0);
    assert_eq!(img_rgb.get_mode(), enums::ImageMode::U12BIT);
    img_rgb.normalize_to_8bit_with_max(2033.0);
    assert_eq!(img_rgb.get_mode(), enums::ImageMode::U8BIT);
}

#[test]
fn test_cropping() {
    let mut img_rgb = rgbimage::RgbImage::open(&String::from(M20_ZCAM_ECM_RGB)).unwrap();
    assert_eq!(img_rgb.width, 1648);
    assert_eq!(img_rgb.height, 1200);
    img_rgb.crop(24, 4, 1600, 1192);
    assert_eq!(img_rgb.width, 1600);
    assert_eq!(img_rgb.height, 1192);
}
