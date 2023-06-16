use crate::guassianblur::guassian_blur_nband;
use crate::image::Image;
use crate::imagebuffer::ImageBuffer;
use anyhow::Result;

pub fn unsharp_mask_nbands(
    buffers: &mut [ImageBuffer],
    sigma: f32,
    amount: f32,
) -> Result<Vec<ImageBuffer>> {
    //FIXME: Unwraps :(
    match guassian_blur_nband(buffers, sigma) {
        Ok(blurred) => Ok((0..blurred.len())
            .map(|b| {
                buffers[b]
                    .add(
                        &buffers[b]
                            .subtract(&blurred[b])
                            .unwrap()
                            .scale(amount)
                            .unwrap(),
                    )
                    .unwrap()
            })
            .collect()),
        Err(why) => Err(why),
    }
}

pub trait RgbImageUnsharpMask {
    fn unsharp_mask(&mut self, sigma: f32, amount: f32);
}

impl RgbImageUnsharpMask for Image {
    fn unsharp_mask(&mut self, sigma: f32, amount: f32) {
        let mut buffers: Vec<ImageBuffer> = (0..self.num_bands())
            .map(|b| self.get_band(b).to_owned())
            .collect();

        if let Ok(buffers) = unsharp_mask_nbands(&mut buffers, sigma, amount) {
            buffers.into_iter().enumerate().for_each(|(b, buffer)| {
                self.set_band(&buffer, b);
            });
        }
    }
}
