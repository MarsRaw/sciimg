use crate::error;
use crate::image::Image;
use crate::imagebuffer::ImageBuffer;
use crate::max;
use crate::Dn;
use crate::DnVec;
use crate::VecMath;

//  SSSSSLLLLOOOOOOWWWWWWW.....
pub fn guassian_blur_nband(
    buffers: &Vec<ImageBuffer>,
    sigma: f32,
) -> error::Result<Vec<ImageBuffer>> {
    if buffers.is_empty() {
        return Err("No buffers provided");
    }

    let radius = max!((3.0 * sigma).ceil(), 1.0) as usize;

    let kernel_length = radius * 2 + 1;

    let mut kernel = DnVec::zeros(kernel_length);
    let mut sum = 0.0;

    let r = radius as i32;

    for i in -r..r {
        let exponent_numerator = -(i * i) as Dn;
        let exponent_denominator = 2.0 * sigma * sigma;

        let e_expression =
            (std::f32::consts::E as Dn).powf(exponent_numerator / exponent_denominator);
        let kernel_value = e_expression / 2.0 * std::f32::consts::PI * sigma * sigma;

        kernel[(i + r) as usize] = kernel_value;
        sum += kernel_value;
    }

    // Normalize kernel
    for (i, _) in kernel.clone().iter().enumerate() {
        kernel[i] /= sum;
    }

    let mut out_buffers = buffers.clone();

    let buffer_width = buffers[0].width;
    let buffer_height = buffers[0].height;

    // 1st pass: Horizontal Blur
    for x in 0..buffer_width {
        for y in 0..buffer_height {
            let mut values = DnVec::zeros(buffers.len());

            for kernel_i in -r..r {
                // Protect image bounds
                if x as i32 - kernel_i < 0 || x as i32 - kernel_i >= buffer_width as i32 {
                    continue;
                }

                let kernel_value = kernel[(kernel_i + r) as usize];

                for b in 0..buffers.len() {
                    values[b] +=
                        out_buffers[b].get(x - kernel_i as usize, y).unwrap() * kernel_value;
                }
            }

            for i in 0..out_buffers.len() {
                out_buffers[i].put(x, y, values[i]);
            }
        }
    }

    let buffers = out_buffers.clone();
    let mut out_buffers = buffers.clone();

    // 2nd pass: Vertical Blur
    for x in 0..buffer_width {
        for y in 0..buffer_height {
            let mut values = DnVec::zeros(buffers.len());

            for kernel_i in -r..r {
                // Protect image bounds
                if y as i32 - kernel_i < 0 || y as i32 - kernel_i >= buffer_height as i32 {
                    continue;
                }

                let kernel_value = kernel[(kernel_i + r) as usize];
                for b in 0..buffers.len() {
                    values[b] +=
                        out_buffers[b].get(x, y - kernel_i as usize).unwrap() * kernel_value;
                }
            }

            for i in 0..out_buffers.len() {
                out_buffers[i].put(x, y, values[i]);
            }
        }
    }
    Ok(out_buffers)
}

pub trait RgbImageBlur {
    fn guassian_blur(&mut self, sigma: f32);
}

impl RgbImageBlur for Image {
    fn guassian_blur(&mut self, sigma: f32) {
        let mut buffers = vec![];
        for b in 0..self.num_bands() {
            buffers.push(self.get_band(b).to_owned());
        }

        if let Ok(buffers) = guassian_blur_nband(&buffers, sigma) {
            for (b, _) in buffers.iter().enumerate() {
                self.set_band(&buffers[b], b);
            }
        }
    }
}
