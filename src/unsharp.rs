use crate::error;
use crate::guassianblur::guassian_blur_nband;
use crate::image::Image;
use crate::imagebuffer::ImageBuffer;

pub fn unsharp_mask_nbands(
    buffers: &Vec<ImageBuffer>,
    sigma: f32,
    amount: f32,
) -> error::Result<Vec<ImageBuffer>> {
    match guassian_blur_nband(buffers, sigma) {
        Ok(blurred) => {
            let mut out_buffers: Vec<ImageBuffer> = vec![];
            for b in 0..blurred.len() {
                out_buffers.push(
                    buffers[b]
                        .add(
                            &buffers[b]
                                .subtract(&blurred[b])
                                .unwrap()
                                .scale(amount)
                                .unwrap(),
                        )
                        .unwrap(),
                );
            }

            Ok(out_buffers)
        }
        Err(why) => Err(why),
    }
}

pub trait RgbImageUnsharpMask {
    fn unsharp_mask(&mut self, sigma: f32, amount: f32);
}

impl RgbImageUnsharpMask for Image {
    fn unsharp_mask(&mut self, sigma: f32, amount: f32) {
        let mut buffers = vec![];
        for b in 0..self.num_bands() {
            buffers.push(self.get_band(b).to_owned());
        }

        if let Ok(buffers) = unsharp_mask_nbands(&buffers, sigma, amount) {
            for (b, _) in buffers.iter().enumerate() {
                self.set_band(&buffers[b], b);
            }
        }
    }
}
