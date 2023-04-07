use crate::error;
use crate::image::Image;
use crate::imagebuffer::ImageBuffer;
use crate::max;
use crate::Dn;
use crate::DnVec;
use crate::VecMath;

#[cfg(rayon)]
use rayon::prelude::*;
#[cfg(rayon)]
use std::sync::Arc;
#[cfg(rayon)]
use std::sync::Mutex;

#[cfg(not(rayon))]
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
    let buff_len: usize = buffers.len();

    // 1st pass: Horizontal Blur
    (0..buffer_width).for_each(|x| {
        (0..buffer_height).for_each(|y| {
            let mut values = DnVec::zeros(buff_len);

            (-r..r).for_each(|kernel_i| {
                // Protect image bounds
                if x as i32 - kernel_i < 0 || x as i32 - kernel_i >= buffer_width as i32 {
                    let kernel_value = kernel[(kernel_i + r) as usize];

                    (0..buff_len).for_each(|b| {
                        values[b] +=
                            buffers[b].get(x - kernel_i as usize, y).unwrap() * kernel_value;
                    });
                }
            });

            (0..buff_len).for_each(|i| {
                buffers[i].put(x, y, values[i]);
            });
        });
    });

    // 2nd pass: Vertical Blur
    (0..buffer_width).for_each(|x| {
        (0..buffer_height).for_each(|y| {
            let mut values = DnVec::zeros(buff_len);

            (-r..r).for_each(|kernel_i| {
                // Protect image bounds
                if y as i32 - kernel_i < 0 || y as i32 - kernel_i >= buffer_height as i32 {
                    let kernel_value = kernel[(kernel_i + r) as usize];
                    (0..buff_len).for_each(|b| {
                        //FIXME: unsafe unwrap
                        values[b] +=
                            buffers[b].get(x, y - kernel_i as usize).unwrap() * kernel_value;
                    });
                }
            });

            (0..buff_len).for_each(|i| {
                buffers[i].put(x, y, values[i]);
            });
        });
    });
    Ok(buffers.into())
}

#[cfg(rayon)]
//Hopefully a little faster?, trivial optimisations.
//There's no test for this one so, we'll see how we go..
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

    for i in -r..r {
        let exponent_numerator = -(i * i) as Dn;
        let exponent_denominator = sig_squared.powi(2);

        let e_expression =
            (std::f32::consts::E as Dn).powf(exponent_numerator / exponent_denominator);

        let kernel_value = e_expression / std::f32::consts::TAU * sig_squared;

        kernel[(i + r) as usize] = kernel_value;
        sum += kernel_value;
    }

    // Normalize kernel
    kernel.par_iter_mut().for_each(|i| {
        *i /= sum;
    });

    // Setup some paralelle iterators, reusable epsilons and locks.
    let buffer_width = buffers[0].width;
    let buffer_height = buffers[0].height;
    let buff_len: usize = buffers.len();

    let width_iter = (0..buffer_width).into_par_iter();
    let height_iter = (0..buffer_height).into_iter();

    // Smart mutually exclusive smart pointer to allow for us to mutate the buffer across threads.
    let m_buffers = Arc::new(Mutex::new(buffers.to_vec()));

    // Without a test to run against it's hard to know if this will help, or just thrash heaps of mutex contention.
    // in theory if the indexes into the buffer are guaranteed to be unique we shouldn't need locks at all..
    // I will probs refactor this to an mpsc pattern to try that next.

    // 1st pass: Horizontal Blur
    width_iter.clone().for_each(|x| {
        let m_c_buffers = m_buffers.clone(); // These are only clones of the pointer

        height_iter.clone().for_each(|y| {
            let mut values = DnVec::zeros(buff_len);

            (-r..r).for_each(|kernel_i| {
                // Protect image bounds
                if x as i32 - kernel_i < 0 || x as i32 - kernel_i >= buffer_width as i32 {
                    let kernel_value = kernel[(kernel_i + r) as usize];

                    (0..buff_len).for_each(|b| {
                        values[b] +=
                            buffers[b].get(x - kernel_i as usize, y).unwrap() * kernel_value;
                    });
                }
            });

            (0..buff_len).for_each(|i| {
                let mut buffers = m_c_buffers.lock().unwrap(); // Move mutable borrow outside of inner closure
                buffers[i].put(x, y, values[i]);
            });
        });
    });

    // 2nd pass: Vertical Blur
    width_iter.for_each(|x| {
        let m_c_buffers = m_buffers.clone(); // These are only clones of the pointer

        height_iter.clone().for_each(|y| {
            let mut values = DnVec::zeros(buff_len);
            (-r..r).for_each(|kernel_i| {
                // Protect image bounds
                if y as i32 - kernel_i < 0 || y as i32 - kernel_i >= buffer_height as i32 {
                    let kernel_value = kernel[(kernel_i + r) as usize];
                    (0..buff_len).for_each(|b| {
                        //FIXME: unsafe unwrap
                        values[b] +=
                            buffers[b].get(x, y - kernel_i as usize).unwrap() * kernel_value;
                    });
                }
            });

            (0..buff_len).for_each(|i| {
                let mut buffers = m_c_buffers.lock().unwrap(); // Move mutable borrow outside of inner closure
                buffers[i].put(x, y, values[i]);
            });
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
