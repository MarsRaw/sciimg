use crate::{image, imagebuffer, lowpass, stats};
use anyhow::{anyhow, Result};

fn apply_blur(image: &imagebuffer::ImageBuffer, amount: usize) -> imagebuffer::ImageBuffer {
    lowpass::lowpass_imagebuffer(image, amount)
}

pub fn get_point_quality_estimation_on_diff_buffer(
    diff: &imagebuffer::ImageBuffer,
    window_size: usize,
    x: usize,
    y: usize,
) -> Result<f32> {
    let window = diff.isolate_window(window_size, x, y);
    match stats::std_deviation(&window) {
        Some(v) => Ok(v),
        None => Err(anyhow!("Failed to generate standard deviation")),
    }
}

pub fn get_point_quality_estimation_on_buffer(
    image: &imagebuffer::ImageBuffer,
    window_size: usize,
    x: usize,
    y: usize,
) -> Result<f32> {
    let blurred = apply_blur(image, 5);
    let diff = blurred.subtract(image)?;
    get_point_quality_estimation_on_diff_buffer(&diff, window_size, x, y)
}

pub fn get_point_quality_estimation(
    image: &image::Image,
    window_size: usize,
    x: usize,
    y: usize,
) -> Result<f32> {
    let mut q: Vec<f32> = vec![];
    for b in 0..image.num_bands() {
        let band = image.get_band(b);
        q.push(get_point_quality_estimation_on_buffer(
            band,
            window_size,
            x,
            y,
        )?);
    }
    match stats::mean(&q) {
        Some(v) => Ok(v),
        None => Err(anyhow!("Failed to generate point quality estimation")),
    }
}

pub fn get_quality_estimation_on_buffer(image: &imagebuffer::ImageBuffer) -> Result<f32> {
    let blurred = apply_blur(image, 5);
    let diff = blurred.subtract(image)?;
    match stats::std_deviation(&diff.buffer.to_vector()) {
        Some(v) => Ok(v),
        None => Err(anyhow!("Failed to generate standard deviation")),
    }
}

// A very simple image sharpness quantifier that computes the standard deviation of the difference between
// an image and a blurred copy.
pub fn get_quality_estimation(image: &image::Image) -> Result<f32> {
    let mut q: Vec<f32> = vec![];
    for b in 0..image.num_bands() {
        let band = image.get_band(b);
        q.push(get_quality_estimation_on_buffer(band)?);
    }
    match stats::mean(&q) {
        Some(v) => Ok(v),
        None => Err(anyhow!("Failed to generate quality estimation")),
    }
}
