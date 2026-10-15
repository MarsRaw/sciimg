use crate::max;
use crate::{image::Image, imagebuffer::ImageBuffer, Dn, DnVec, VecMath};
use anyhow::{anyhow, Result};

/// NOTE: this will call the parallelized version if you are using the 'rayon' feature.
pub fn gaussian_blur_nband(buffers: &mut [ImageBuffer], sigma: f32) -> Result<Vec<ImageBuffer>> {
    #[cfg(not(feature = "rayon"))]
    {
        st_gaussian_blur(buffers, sigma)
    }
    #[cfg(feature = "rayon")]
    {
        par_gaussian_blur(buffers, sigma)
    }
}

pub trait RgbImageBlur {
    fn gaussian_blur(&mut self, sigma: f32);
}

impl RgbImageBlur for Image {
    fn gaussian_blur(&mut self, sigma: f32) {
        let mut buffers = vec![];
        (0..self.num_bands()).for_each(|b| {
            buffers.push(self.get_band(b).to_owned());
        });

        if let Ok(buffers) = gaussian_blur_nband(&mut buffers, sigma) {
            buffers.iter().enumerate().for_each(|(b, _)| {
                self.set_band(&buffers[b], b);
            });
        }
    }
}

#[cfg(feature = "rayon")]
pub fn par_gaussian_blur(
    buffers: &[ImageBuffer],
    sigma: f32,
) -> Result<Vec<ImageBuffer>, anyhow::Error> {
    use rayon::prelude::*;

    if buffers.is_empty() {
        return Err(anyhow!("No buffers provided"));
    }

    let sig_squared = sigma.powi(2);
    let radius = (3.0 * sigma).ceil().max(1.0) as usize;
    let kernel_length = radius * 2 + 1;
    let mut kernel = DnVec::zeros(kernel_length);
    let mut sum = 0.0;
    let r = radius as i32;

    (-r..=r).for_each(|i| {
        let exponent = -(i * i) as Dn / (2.0 * sig_squared);
        let kernel_value =
            (std::f32::consts::E as Dn).powf(exponent) / (std::f32::consts::TAU * sig_squared);
        kernel[(i + r) as usize] = kernel_value;
        sum += kernel_value;
    });
    kernel.iter_mut().for_each(|v| *v /= sum);

    let buffer_width = buffers[0].width;
    let buffer_height = buffers[0].height;

    // Horizontal pass: each buffer independently.
    let mut horizontal_buffers = buffers.iter().cloned().collect::<Vec<_>>();
    horizontal_buffers
        .par_iter_mut()
        .enumerate()
        .for_each(|(b, buf)| {
            for y in 0..buffer_height {
                for x in 0..buffer_width {
                    let mut acc = 0.0;
                    for kernel_i in -r..=r {
                        let x_sample = x as i32 + kernel_i; // using addition
                        if x_sample < 0 || x_sample >= buffer_width as i32 {
                            continue;
                        }
                        let v = match buffers[b].safe_get(x_sample as usize, y) {
                            Some(val) => val,
                            None => continue, // skip if out of bounds
                        };
                        acc += v * kernel[(kernel_i + r) as usize];
                    }
                    buf.put(x, y, acc);
                }
            }
        });

    // Vertical pass: each buffer independently.
    let mut vertical_buffers = horizontal_buffers.clone();
    vertical_buffers
        .par_iter_mut()
        .enumerate()
        .for_each(|(b, buf)| {
            for x in 0..buffer_width {
                for y in 0..buffer_height {
                    let mut acc = 0.0;
                    for kernel_i in -r..=r {
                        let y_sample = y as i32 + kernel_i; // using addition
                        if y_sample < 0 || y_sample >= buffer_height as i32 {
                            continue;
                        }
                        let v = match horizontal_buffers[b].safe_get(x, y_sample as usize) {
                            Some(val) => val,
                            None => continue,
                        };
                        acc += v * kernel[(kernel_i + r) as usize];
                    }
                    buf.put(x, y, acc);
                }
            }
        });

    Ok(vertical_buffers)
}

/// SLOW (according to Kevin...)
pub fn st_gaussian_blur(buffers: &mut [ImageBuffer], sigma: f32) -> Result<Vec<ImageBuffer>> {
    if buffers.is_empty() {
        return Err(anyhow!("No buffers provided"));
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
    let buff_len: usize = buffers.len();

    // Horizontal pass
    (0..buffer_width).for_each(|x| {
        (0..buffer_height).for_each(|y| {
            let mut values = DnVec::zeros(buff_len);
            'h_kernel: for kernel_i in -r..=r {
                let x_sample = x as i32 - kernel_i;
                let k = kernel[(kernel_i + r) as usize];
                for b in 0..buff_len {
                    let v = match buffers[b].safe_get(x_sample as usize, y) {
                        Some(val) => val,
                        None => continue 'h_kernel,
                    };
                    values[b] += v * k;
                }
            }
            for i in 0..buff_len {
                buffers[i].put(x, y, values[i]);
            }
        });
    });

    // Vertical pass
    (0..buffer_width).for_each(|x| {
        (0..buffer_height).for_each(|y| {
            let mut values = DnVec::zeros(buff_len);
            'v_kernel: for kernel_i in -r..=r {
                let y_sample = y as i32 - kernel_i;
                let k = kernel[(kernel_i + r) as usize];
                for b in 0..buff_len {
                    let v = match buffers[b].safe_get(x, y_sample as usize) {
                        Some(val) => val,
                        None => continue 'v_kernel,
                    };
                    values[b] += v * k;
                }
            }
            for i in 0..buff_len {
                buffers[i].put(x, y, values[i]);
            }
        });
    });

    Ok(buffers.into())
}
