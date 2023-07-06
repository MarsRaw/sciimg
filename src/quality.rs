use crate::{image, imagebuffer, lowpass, stats};

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
    stats::std_deviation(&window)
}

pub fn get_point_quality_estimation_on_buffer(
    image: &imagebuffer::ImageBuffer,
    window_size: usize,
    x: usize,
    y: usize,
) -> f32 {
    let subframe = image
        .get_subframe(
            x - window_size / 2,
            y - window_size / 2,
            window_size,
            window_size,
        )
        .expect("Failed to extract subframe");

    let blurred = apply_blur(&subframe, 5);
    let diff = blurred.subtract(&subframe).unwrap();
    get_point_quality_estimation_on_diff_buffer(
        &diff,
        window_size,
        window_size / 2,
        window_size / 2,
    )
}

pub fn get_point_quality_estimation(
    image: &image::Image,
    window_size: usize,
    x: usize,
    y: usize,
) -> f32 {
    let q: Vec<f32> = (0..image.num_bands())
        .map(|b| get_point_quality_estimation_on_buffer(image.get_band(b), window_size, x, y))
        .collect::<Vec<f32>>();
    stats::mean(&q)
}

pub fn get_quality_estimation_on_buffer(image: &imagebuffer::ImageBuffer) -> f32 {
    let blurred = apply_blur(image, 5);
    let diff = blurred.subtract(image).unwrap();
    stats::std_deviation(&diff.buffer.to_vector())
}

// A very simple image sharpness quantifier that computes the standard deviation of the difference between
// an image and a blurred copy.
pub fn get_quality_estimation(image: &image::Image) -> f32 {
    let q: Vec<f32> = (0..image.num_bands())
        .map(|b| get_quality_estimation_on_buffer(image.get_band(b)))
        .collect::<Vec<f32>>();
    stats::mean(&q)
}
