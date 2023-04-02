use crate::prelude::*;
use imageproc::filter::median_filter;

pub fn median_blur(image: &ImageBuffer, radius: usize) -> ImageBuffer {
    // Would be lovely if there were a 16bit implementation of median_filter from imageproc
    let ib = image.normalize(0.0, 255.0).unwrap().buffer_to_image_8bit();
    let filtered = median_filter(&ib, radius as u32, radius as u32);
    ImageBuffer::from_image_u8(&filtered)
        .unwrap()
        .normalize(0.0, 65535.0)
        .unwrap()
}
