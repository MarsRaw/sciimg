use crate::{imagebuffer, lowpass, rgbimage, stats};

fn apply_blur(image: &imagebuffer::ImageBuffer, amount: usize) -> imagebuffer::ImageBuffer {
    lowpass::lowpass_imagebuffer(image, amount)
}

pub fn get_point_quality_estimation_on_diff_buffer(
    diff: &imagebuffer::ImageBuffer,
    window_size: usize,
    x: usize,
    y: usize,
) -> f32 {
    let window = diff.isolate_window(window_size, x, y);
    stats::std_deviation(&window).unwrap_or(0.0)
}

pub fn get_point_quality_estimation_on_buffer(
    image: &imagebuffer::ImageBuffer,
    window_size: usize,
    x: usize,
    y: usize,
) -> f32 {
    let blurred = apply_blur(image, 5);
    let diff = blurred.subtract(image).unwrap();
    get_point_quality_estimation_on_diff_buffer(&diff, window_size, x, y)
}

pub fn get_point_quality_estimation(
    image: &rgbimage::RgbImage,
    window_size: usize,
    x: usize,
    y: usize,
) -> f32 {
    let mut q: Vec<f32> = vec![];
    for b in 0..image.num_bands() {
        let band = image.get_band(b);
        q.push(get_point_quality_estimation_on_buffer(
            band,
            window_size,
            x,
            y,
        ));
    }
    stats::mean(&q).unwrap_or(0.0)
}

pub fn get_quality_estimation_on_buffer(image: &imagebuffer::ImageBuffer) -> f32 {
    let blurred = apply_blur(image, 5);
    let diff = blurred.subtract(image).unwrap();
    stats::std_deviation(&diff.buffer.to_vector()).unwrap_or(0.0)
}

// A very simple image sharpness quantifier that computes the standard deviation of the difference between
// an image and a blurred copy.
pub fn get_quality_estimation(image: &rgbimage::RgbImage) -> f32 {
    let mut q: Vec<f32> = vec![];
    for b in 0..image.num_bands() {
        let band = image.get_band(b);
        q.push(get_quality_estimation_on_buffer(band));
    }
    stats::mean(&q).unwrap_or(0.0)
}
