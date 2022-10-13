use image::{imageops::blur, DynamicImage, Rgba};

use crate::imagebuffer::ImageBuffer;

pub fn blur_vec_u8(v: &Vec<u8>, width: usize, height: usize, amount: f32) -> Vec<u8> {
    let mut out_img = DynamicImage::new_rgba8(width as u32, height as u32).into_rgba16();
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            out_img.put_pixel(
                x as u32,
                y as u32,
                Rgba([v[i] as u16, v[i] as u16, v[i] as u16, 255]),
            );
        }
    }
    let blurred = blur(&out_img, amount);

    let mut blurred_v: Vec<u8> = Vec::with_capacity(width * height);
    blurred_v.resize(width * height, 0);

    for y in 0..height {
        for x in 0..width {
            let pixel = blurred.get_pixel(x as u32, y as u32);
            let value = pixel[0] as u8;
            let idx = y * width + x;
            blurred_v[idx] = value;
        }
    }

    blurred_v
}

pub fn blur_vec_u16(v: &Vec<u16>, width: usize, height: usize, amount: f32) -> Vec<u16> {
    let mut out_img = DynamicImage::new_rgba16(width as u32, height as u32).into_rgba16();
    for y in 0..height {
        for x in 0..width {
            let i = y * width + x;
            out_img.put_pixel(
                x as u32,
                y as u32,
                Rgba([v[i] as u16, v[i] as u16, v[i] as u16, 65535]),
            );
        }
    }
    let blurred = blur(&out_img, amount);

    let mut blurred_v: Vec<u16> = Vec::with_capacity(width * height);
    blurred_v.resize(width * height, 0);

    for y in 0..height {
        for x in 0..width {
            let pixel = blurred.get_pixel(x as u32, y as u32);
            let value = pixel[0] as u16;
            let idx = y * width + x;
            blurred_v[idx] = value;
        }
    }

    blurred_v
}

pub fn blur_imagebuffer(imagebuff: &ImageBuffer, amount: f32) -> ImageBuffer {
    // fastblur::gaussian_blur only supports vectors of u8 rgb. So we are forced
    // to scale to that then scale back to f32... This is quite lossy.

    let v_u16 = imagebuff.to_vector_u16();
    let blurred = blur_vec_u16(&v_u16, imagebuff.width, imagebuff.height, amount);
    ImageBuffer::from_vec_u16_with_mask(
        &blurred,
        imagebuff.width,
        imagebuff.height,
        &imagebuff.to_mask(),
    )
    .unwrap()
}
