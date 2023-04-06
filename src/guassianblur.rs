use image::buffer;

use crate::error;
use crate::image::Image;
use crate::imagebuffer::ImageBuffer;
use crate::max;
use crate::Dn;
use crate::DnVec;
use crate::VecMath;

//  SSSSSLLLLOOOOOOWWWWWWW.....
pub fn guassian_blur_nband(
    buffers: &mut [ImageBuffer],
    sigma: f32,
) -> error::Result<Vec<ImageBuffer>> {
    if buffers.is_empty() {
        return Err("No buffers provided");
    }

    let sig_squared = sigma.powi(2);
    let radius = max!((3.0 * sigma).ceil(), 1.0) as usize;

    let kernel_length = radius * 2 + 1;

    let mut kernel = DnVec::zeros(kernel_length);
    let mut sum = 0.0;

    let r = radius as i32;

    (-r..r).for_each(|i| {
        let exponent_numerator = -(i * i) as Dn;
        let exponent_denominator = 2.0 * sig_squared;

        let e_expression =
            (std::f32::consts::E as Dn).powf(exponent_numerator / exponent_denominator);
        let kernel_value = e_expression / std::f32::consts::TAU * sig_squared;

        kernel[(i + r) as usize] = kernel_value;
        sum += kernel_value;
    });

    // Normalize kernel
    kernel.iter_mut().for_each(|i| {
        *i /= sum;
    });

    let buffer_width = buffers[0].width;
    let buffer_height = buffers[0].height;

    // 1st pass: Horizontal Blur
    (0..buffer_width).for_each(|x| {
        (0..buffer_height).for_each(|y| {
            let buff_len: usize = buffers.len();
            let mut values = DnVec::zeros(buff_len);

            for kernel_i in -r..r {
                // Protect image bounds
                if x as i32 - kernel_i < 0 || x as i32 - kernel_i >= buffer_width as i32 {
                    continue;
                }

                let kernel_value = kernel[(kernel_i + r) as usize];

                (0..buff_len).for_each(|b| {
                    values[b] += buffers[b].get(x - kernel_i as usize, y).unwrap() * kernel_value;
                });
            }

            (0..buff_len).for_each(|i| {
                buffers[i].put(x, y, values[i]);
            });
        });
    });

    // 2nd pass: Vertical Blur
    (0..buffer_width).for_each(|x| {
        (0..buffer_height).for_each(|y| {
            let mut values = DnVec::zeros(buffers.len());

            for kernel_i in -r..r {
                // Protect image bounds
                if y as i32 - kernel_i < 0 || y as i32 - kernel_i >= buffer_height as i32 {
                    continue;
                }

                let kernel_value = kernel[(kernel_i + r) as usize];
                (0..buffers.len()).for_each(|b| {
                    //FIXME: unsafe unwrap
                    values[b] += buffers[b].get(x, y - kernel_i as usize).unwrap() * kernel_value;
                });
            }

            for i in 0..buffers.len() {
                buffers[i].put(x, y, values[i]);
            }
        });
    });
    Ok(buffers.into())
}

pub trait RgbImageBlur {
    fn guassian_blur(&mut self, sigma: f32);
}

impl RgbImageBlur for Image {
    fn guassian_blur(&mut self, sigma: f32) {
        let mut buffers = vec![];
        (0..self.num_bands()).for_each(|b| {
            buffers.push(self.get_band(b).to_owned());
        });

        if let Ok(buffers) = guassian_blur_nband(&mut buffers, sigma) {
            buffers.iter().enumerate().for_each(|(b, _)| {
                self.set_band(&buffers[b], b);
            });
        }
    }
}
