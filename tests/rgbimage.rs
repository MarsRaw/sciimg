use sciimg::{decompanding, enums, enums::ImageMode, imagebuffer::ImageBuffer, rgbimage};

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

#[test]
fn test_rgbimage_math_3band() {
    let b0 = ImageBuffer::new_with_fill(1000, 1000, 100.0).unwrap();

    let mut img =
        rgbimage::RgbImage::new_from_buffers_rgb(&b0, &b0, &b0, ImageMode::U16BIT).unwrap();

    let img2 = rgbimage::RgbImage::new_from_buffers_rgb(&b0, &b0, &b0, ImageMode::U16BIT).unwrap();
    assert_eq!(img.width, 1000);
    assert_eq!(img.height, 1000);
    assert_eq!(img.num_bands(), 3);

    img.add(&img2);
    assert_eq!(img.width, 1000);
    assert_eq!(img.height, 1000);
    assert_eq!(img.num_bands(), 3);
    assert_eq!(img.get_band(0).get(100, 100).unwrap(), 200.0);
    assert_eq!(img.get_band(1).get(100, 100).unwrap(), 200.0);
    assert_eq!(img.get_band(2).get(100, 100).unwrap(), 200.0);

    img.apply_weight_on_band(0.1, 0);
    assert_eq!(img.get_band(0).get(100, 100).unwrap(), 20.0);
    assert_eq!(img.get_band(1).get(100, 100).unwrap(), 200.0);
    assert_eq!(img.get_band(2).get(100, 100).unwrap(), 200.0);

    img.divide_from_each(&b0);
    assert_eq!(img.get_band(0).get(100, 100).unwrap(), 0.2);
    assert_eq!(img.get_band(1).get(100, 100).unwrap(), 2.0);
    assert_eq!(img.get_band(2).get(100, 100).unwrap(), 2.0);
}

#[test]
fn test_rgbimage_math_1band() {
    let b0 = ImageBuffer::new_with_fill(1000, 1000, 100.0).unwrap();
    let b1 = ImageBuffer::new_with_fill(1000, 1000, 200.0).unwrap();

    let mut img = rgbimage::RgbImage::new_with_bands(1000, 1000, 1, ImageMode::U16BIT).unwrap();
    img.set_band(&b0, 0);

    let mut img2 = rgbimage::RgbImage::new_with_bands(1000, 1000, 1, ImageMode::U16BIT).unwrap();
    img2.set_band(&b0, 0);

    let mut img3 = rgbimage::RgbImage::new_with_bands(1000, 1000, 1, ImageMode::U16BIT).unwrap();
    img3.set_band(&b1, 0);

    assert_eq!(img.width, 1000);
    assert_eq!(img.height, 1000);
    assert_eq!(img.num_bands(), 1);

    img.add(&img2);
    assert_eq!(img.width, 1000);
    assert_eq!(img.height, 1000);
    assert_eq!(img.num_bands(), 1);
    assert_eq!(img.get_band(0).get(100, 100).unwrap(), 200.0);

    img.add(&img2);
    assert_eq!(img.width, 1000);
    assert_eq!(img.height, 1000);
    assert_eq!(img.num_bands(), 1);
    assert_eq!(img.get_band(0).get(100, 100).unwrap(), 300.0);

    img.add(&img3);
    assert_eq!(img.width, 1000);
    assert_eq!(img.height, 1000);
    assert_eq!(img.num_bands(), 1);
    assert_eq!(img.get_band(0).get(100, 100).unwrap(), 500.0);

    img.apply_weight_on_band(0.1, 0);
    assert_eq!(img.get_band(0).get(100, 100).unwrap(), 50.0);

    img.divide_from_each(&b0);
    assert_eq!(img.get_band(0).get(100, 100).unwrap(), 0.5);
}
