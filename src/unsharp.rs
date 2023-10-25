use crate::guassianblur::gaussian_blur_2d_nbands;
use crate::image::Image;
use crate::imagebuffer::ImageBuffer;

pub fn unsharp_mask_nbands(buffers: &[ImageBuffer], sigma: f32, amount: f32) -> Vec<ImageBuffer> {
    gaussian_blur_2d_nbands(buffers, sigma)
        .into_iter()
        .enumerate()
        .map(|(i, b)| {
            let mm0 = buffers[i].get_min_max();
            let mm1 = b.get_min_max();
            println!("mm0: {:?}  -- mm1: {:?}", mm0, mm1);

            let c = b.normalize(mm0.min, mm0.max).unwrap();
            buffers[i]
                .add(&buffers[i].subtract(&c).unwrap().scale(amount).unwrap())
                .unwrap()
        })
        .collect()
}

pub fn unsharp_image(image: &Image, sigma: f32, amount: f32) -> Image {
    let mut unsharped_image = image.clone();
    unsharped_image.bands = unsharp_mask_nbands(&unsharped_image.bands, sigma, amount);
    unsharped_image
}

pub trait RgbImageUnsharpMask {
    fn unsharp_mask(&mut self, sigma: f32, amount: f32);
}

impl RgbImageUnsharpMask for Image {
    fn unsharp_mask(&mut self, sigma: f32, amount: f32) {
        self.bands = unsharp_mask_nbands(&self.bands, sigma, amount);
    }
}
