use crate::image::Image;
use anyhow::Result;

fn remove_local_mean(g: &mut [f32], w: usize, h: usize, r: usize) {
    let integral = Integral::new(g, w, h);
    for y in 0..h {
        for x in 0..w {
            g[y * w + x] -= integral.box_mean(w, h, x, y, r) as f32;
        }
    }
}

pub struct Integral {
    pub data: Vec<f32>,
    pub w: usize,
    pub sum: Vec<f64>,
    pub sq: Vec<f64>,
}

impl Integral {
    pub fn new(data: &[f32], w: usize, h: usize) -> Self {
        let iw = w + 1;
        let mut sum = vec![0.0; iw * (h + 1)];
        let mut sq = vec![0.0; iw * (h + 1)];
        for y in 0..h {
            let (mut rs, mut rq) = (0.0, 0.0);
            for x in 0..w {
                let v = data[y * w + x] as f64;
                rs += v;
                rq += v * v;
                sum[(y + 1) * iw + x + 1] = sum[y * iw + x + 1] + rs;
                sq[(y + 1) * iw + x + 1] = sq[y * iw + x + 1] + rq;
            }
        }
        Self {
            data: data.to_vec(),
            w: iw,
            sum,
            sq,
        }
    }

    // (sum, sum of squares) over the k×k window with top-left (x, y)
    pub fn window(&self, x: usize, y: usize, k: usize) -> (f64, f64) {
        let f = |t: &[f64]| {
            t[(y + k) * self.w + x + k] - t[y * self.w + x + k] - t[(y + k) * self.w + x]
                + t[y * self.w + x]
        };
        (f(&self.sum), f(&self.sq))
    }

    /// Mean over the (2r+1)^2 window at (x, y), clamped to the image.
    pub fn box_mean(&self, w: usize, h: usize, x: usize, y: usize, r: usize) -> f64 {
        let stride = w + 1;
        let x0 = x.saturating_sub(r);
        let x1 = (x + r + 1).min(w);
        let y0 = y.saturating_sub(r);
        let y1 = (y + r + 1).min(h);
        let sum =
            self.sum[y1 * stride + x1] - self.sum[y0 * stride + x1] - self.sum[y1 * stride + x0]
                + self.sum[y0 * stride + x0];
        sum / ((x1 - x0) * (y1 - y0)) as f64
    }
}

pub trait ImageIntegral {
    fn to_mono_integral_basic(&self, kernel_size: usize) -> Result<Integral>;
    fn to_mono_integral(
        &self,
        norm_min: f32,
        norm_max: f32,
        remove_means: bool,
        kernel_size: usize,
    ) -> Result<Integral>;
}

impl ImageIntegral for Image {
    fn to_mono_integral_basic(&self, kernel_size: usize) -> Result<Integral> {
        self.to_mono_integral(0.0, 1.0, true, kernel_size)
    }

    fn to_mono_integral(
        &self,
        norm_min: f32,
        norm_max: f32,
        remove_means: bool,
        kernel_size: usize,
    ) -> Result<Integral> {
        let img_mono = self.to_grayscale()?;
        let (lw, lh) = (img_mono.width, img_mono.height);

        let mut b = img_mono.get_band(0).clone();
        b.normalize_mut(norm_min, norm_max);

        let mut px: Vec<f32> = b.to_vector();

        if remove_means {
            remove_local_mean(&mut px, lw, lh, kernel_size);
        }

        Ok(Integral::new(&px, lw, lh))
    }
}
