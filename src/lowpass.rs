use crate::{imagebuffer::ImageBuffer, rgbimage::RgbImage, stats};

fn isolate_window(buffer: &ImageBuffer, window_size: usize, x: usize, y: usize) -> Vec<f32> {
    let mut v: Vec<f32> = Vec::with_capacity(window_size * window_size);
    let start = -(window_size as i32 / 2);
    let end = window_size as i32 / 2 + 1;
    for _y in start..end {
        for _x in start..end {
            let get_x = x as i32 + _x;
            let get_y = y as i32 + _y;
            if get_x >= 0
                && get_x < buffer.width as i32
                && get_y >= 0
                && get_y < buffer.height as i32
                && buffer.get_mask_at_point(get_x as usize, get_y as usize)
            {
                v.push(buffer.get(get_x as usize, get_y as usize).unwrap());
            }
        }
    }
    v
}

fn mean_of_window(buffer: &ImageBuffer, window_size: usize, x: usize, y: usize) -> Option<f32> {
    let window_values = isolate_window(buffer, window_size, x, y);
    stats::mean(&window_values)
}

pub fn lowpass_imagebuffer(imagebuff: &ImageBuffer, window_size: usize) -> ImageBuffer {
    let mut lowpass_buffer =
        ImageBuffer::new_with_mask(imagebuff.width, imagebuff.height, &imagebuff.to_mask())
            .unwrap();

    (0..lowpass_buffer.height).for_each(|y| {
        (0..lowpass_buffer.width).for_each(|x| {
            if let Some(m) = mean_of_window(imagebuff, window_size, x, y) {
                lowpass_buffer.put(x, y, m);
            }
        });
    });

    lowpass_buffer
}

pub fn lowpass(image: &RgbImage, window_size: usize) -> RgbImage {
    let mut lowpass_image = RgbImage::new(image.width, image.height, image.get_mode()).unwrap();

    (0..image.num_bands()).for_each(|b| {
        let buffer = image.get_band(b);
        let filtered_buffer = lowpass_imagebuffer(buffer, window_size);
        lowpass_image.push_band(&filtered_buffer);
    });

    lowpass_image
}
