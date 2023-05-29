use crate::{enums::ImageMode, image::Image, path};

use anyhow::{anyhow, Result};
use image::{DynamicImage, Luma, Rgb, Rgba};

// Note: This much code duplication is ridiculous and should not stay this way

pub fn save_image_to_rgb_16bpp(output_file_name: &str, image: &Image) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if image.num_bands() < 3 {
        return Err(anyhow!(
            "Image contains insufficient bands to produce an RGB output"
        ));
    }
    if image.get_mode() != ImageMode::U16BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }

    let mut out_img = DynamicImage::new_rgb16(image.width as u32, image.height as u32).into_rgb16();

    for y in 0..image.height {
        for x in 0..image.width {
            out_img.put_pixel(
                x as u32,
                y as u32,
                Rgb([
                    image.get_band(0).get(x, y).round() as u16,
                    image.get_band(1).get(x, y).round() as u16,
                    image.get_band(2).get(x, y).round() as u16,
                ]),
            );
        }
    }

    out_img
        .save(output_file_name)
        .expect("Failed to save image");

    Ok(())
}

pub fn save_image_to_rgba_16bpp(output_file_name: &str, image: &Image) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if !image.is_using_alpha() {
        return Err(anyhow!(
            "Image contains insufficient bands to produce an RGBA output"
        ));
    }
    if image.get_mode() != ImageMode::U16BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }

    let mut out_img =
        DynamicImage::new_rgba16(image.width as u32, image.height as u32).into_rgba16();

    for y in 0..image.height {
        for x in 0..image.width {
            out_img.put_pixel(
                x as u32,
                y as u32,
                Rgba([
                    image.get_band(0).get(x, y).round() as u16,
                    image.get_band(1).get(x, y).round() as u16,
                    image.get_band(2).get(x, y).round() as u16,
                    if image.get_alpha_at(x, y) {
                        std::u16::MAX
                    } else {
                        std::u16::MIN
                    },
                ]),
            );
        }
    }

    out_img
        .save(output_file_name)
        .expect("Failed to save image");
    Ok(())
}

pub fn save_image_to_mono_16bpp(
    output_file_name: &str,
    image: &Image,
    use_band: usize,
) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if image.get_mode() != ImageMode::U16BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }
    if image.num_bands() <= use_band {
        return Err(anyhow!(
            "Image does not contain the requested color band: {}",
            use_band
        ));
    }

    let mut out_img =
        DynamicImage::new_luma16(image.width as u32, image.height as u32).into_luma16();

    for y in 0..image.height {
        for x in 0..image.width {
            out_img.put_pixel(
                x as u32,
                y as u32,
                Luma([image.get_band(use_band).get(x, y).round() as u16]),
            );
        }
    }

    out_img
        .save(output_file_name)
        .expect("Failed to save image");

    Ok(())
}

pub fn save_image_to_rgb_8bpp(output_file_name: &str, image: &Image) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if image.num_bands() < 3 {
        return Err(anyhow!(
            "Image contains insufficient bands to produce an RGB output"
        ));
    }
    if image.get_mode() != ImageMode::U8BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }
    let mut out_img = DynamicImage::new_rgb8(image.width as u32, image.height as u32).into_rgb8();

    for y in 0..image.height {
        for x in 0..image.width {
            out_img.put_pixel(
                x as u32,
                y as u32,
                Rgb([
                    image.get_band(0).get(x, y).round() as u8,
                    image.get_band(1).get(x, y).round() as u8,
                    image.get_band(2).get(x, y).round() as u8,
                ]),
            );
        }
    }

    out_img
        .save(output_file_name)
        .expect("Failed to save image");
    Ok(())
}

pub fn save_image_to_rgba_8bpp(output_file_name: &str, image: &Image) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if !image.is_using_alpha() {
        return Err(anyhow!(
            "Image contains insufficient bands to produce an RGBA output"
        ));
    }
    if image.get_mode() != ImageMode::U8BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }

    let mut out_img = DynamicImage::new_rgba8(image.width as u32, image.height as u32).into_rgba8();

    for y in 0..image.height {
        for x in 0..image.width {
            out_img.put_pixel(
                x as u32,
                y as u32,
                Rgba([
                    image.get_band(0).get(x, y).round() as u8,
                    image.get_band(1).get(x, y).round() as u8,
                    image.get_band(2).get(x, y).round() as u8,
                    if image.get_alpha_at(x, y) {
                        std::u8::MAX
                    } else {
                        std::u8::MIN
                    },
                ]),
            );
        }
    }

    out_img
        .save(output_file_name)
        .expect("Failed to save iamge");

    Ok(())
}

pub fn save_image_to_mono_8bpp(
    output_file_name: &str,
    image: &Image,
    use_band: usize,
) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if image.get_mode() != ImageMode::U8BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }
    if image.num_bands() <= use_band {
        return Err(anyhow!(
            "Image does not contain the requested color band: {}",
            use_band
        ));
    }

    let mut out_img = DynamicImage::new_luma8(image.width as u32, image.height as u32).into_luma8();

    for y in 0..image.height {
        for x in 0..image.width {
            out_img.put_pixel(
                x as u32,
                y as u32,
                Luma([image.get_band(use_band).get(x, y).round() as u8]),
            );
        }
    }

    out_img
        .save(output_file_name)
        .expect("Failed to save image");

    Ok(())
}
