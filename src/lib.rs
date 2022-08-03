
pub mod camera;
pub mod debayer;
pub mod decompanding;
pub mod error;
pub mod hotpixel;
pub mod imagebuffer;
pub mod inpaint;
pub mod matrix;
pub mod noise;
pub mod rgbimage;
pub mod stats;
pub mod vector;
pub mod quaternion;
pub mod enums;
pub mod path;
pub mod util;
pub mod imagerot;
pub mod blur;
pub mod lowpass;
pub mod quality;
pub mod prelude;



// Dn -> Digital number / image pixel value as 32 bit floating point.
type Dn = f32;
type DnVec = Vec<Dn>;

pub fn center_crop_2d<T:Copy>(from_array:&Vec<T>, from_width:usize, from_height:usize, to_width:usize, to_height:usize) -> Vec<T> {
    let mut new_arr : Vec<T> = Vec::with_capacity(to_width * to_height);

    for y in 0..to_height {
        for x in 0..to_width {
    
            let from_x = ((from_width - to_width) / 2) + x;
            let from_y = ((from_height - to_height) / 2) + y;
            let from_idx = from_y * from_width + from_x;

            new_arr.push(from_array[from_idx]);
        }
    }
    
    new_arr
}

pub fn crop_2d<T:Copy>(from_array:&Vec<T>, from_width:usize, from_height:usize, left_x:usize, top_y:usize, to_width:usize, to_height:usize) -> Vec<T> {
    if top_y + to_height > from_height || left_x + to_width > from_width {
        panic!("Crop bounds exceeed source array");
    }
    
    let mut new_arr: Vec<T>  = Vec::with_capacity(to_width * to_height);

    for y in 0..to_height {
        for x in 0..to_width {
            let from_idx = (top_y + y) * from_width + (left_x + x);
            new_arr.push(from_array[from_idx]);
        }
    }
    new_arr
}

#[derive(Debug, Clone)]
pub struct MinMax {
    pub min: Dn,
    pub max: Dn,
}

pub trait VecMath {
    fn fill(capacity:usize, fill_value:Dn) -> DnVec;
    fn zeros(capacity:usize) -> DnVec;
    fn sum(&self) -> Dn;
    fn mean(&self) -> Dn;
    fn variance(&self) -> Dn;
    fn xcorr(&self, other:&Self) -> Dn;
    fn stddev(&self) -> Dn;
    fn z_score(&self, check_value:Dn) -> Dn;
    fn isolate_window_2d(&self, width_2d:usize, height_2d:usize, window_size:usize, x:usize, y:usize) -> DnVec;
    fn get_2d(&self, width_2d:usize, height_2d:usize, x:usize, y:usize) -> Dn;
    fn center_crop_2d(&self, from_width:usize, from_height:usize, to_width:usize, to_height:usize) -> DnVec;
    fn crop_2d(&self, from_width:usize, from_height:usize, left_x:usize, top_y:usize, to_width:usize, to_height:usize) -> DnVec;

    fn add(&self, other:&DnVec) -> DnVec;
    fn add_mut(&mut self, other:&DnVec);

    fn add_across(&self, other:Dn) -> DnVec;
    fn add_across_mut(&mut self, other:Dn);

    fn subtract(&self, other:&DnVec) -> DnVec;
    fn subtract_mut(&mut self, other:&DnVec);

    fn subtract_across(&self, other:Dn) -> DnVec;
    fn subtract_across_mut(&mut self, other:Dn);

    fn divide(&self, other:&DnVec) -> DnVec;
    fn divide_mut(&mut self, other:&DnVec);

    fn divide_into(&self, divisor:Dn) -> DnVec;
    fn divide_into_mut(&mut self, divisor:Dn);

    fn scale(&self, scalar:Dn) -> DnVec;
    fn scale_mut(&mut self, scalar:Dn);

    fn multiply(&self, other:&DnVec) -> DnVec;
    fn multiply_mut(&mut self, other:&DnVec);

    fn power(&self, exponent:Dn) -> DnVec;
    fn power_mut(&mut self, exponent:Dn);

    fn clip(&self, clip_min:Dn, clip_max:Dn) -> DnVec;
    fn clip_mut(&mut self, clip_min:Dn, clip_max:Dn);

    fn paste_2d(&self, dest_width:usize, dest_height:usize, src:&DnVec, src_width:usize, src_height:usize, tl_x:usize, tl_y:usize) -> DnVec;
    fn paste_mut_2d(&mut self, dest_width:usize, dest_height:usize, src:&DnVec, src_width:usize, src_height:usize, tl_x:usize, tl_y:usize);

    fn normalize_force_minmax(&self, min:Dn, max:Dn, forced_min:Dn, forced_max:Dn) -> DnVec;
    fn normalize_force_minmax_mut(&mut self, min:Dn, max:Dn, forced_min:Dn, forced_max:Dn);

    fn get_min_max(&self) -> MinMax;

    fn normalize(&self, min:Dn, max:Dn) -> DnVec;
    fn normalize_mut(&mut self, min:Dn, max:Dn);
    
}

impl VecMath for DnVec {

    fn fill(capacity:usize, fill_value:Dn) -> DnVec {
        let mut v:DnVec = Vec::with_capacity(capacity);
        v.resize(capacity, fill_value);
        v
    }

    fn zeros(capacity:usize) -> DnVec {
        DnVec::fill(capacity, 0.0)
    }

    fn sum(&self) -> Dn {
        let mut s = 0.0;
        for v in self.iter() {
            s += v;
        }
        s
    }

    fn mean(&self) -> Dn {
        self.sum() / self.len() as Dn
    }
    
    fn variance(&self) -> Dn {
        let m = self.mean();

        let mut sqdiff = 0.0;
        for v in self.iter() {
            sqdiff += (v - m) * (v - m);
        }
        sqdiff / self.len() as Dn
    }



    fn xcorr(&self, other:&Self) -> Dn {
        if self.len() != other.len() {
            panic!("Arrays need to be the same length (for now)");
        }
        let m_x = self.mean();
        let m_y = other.mean();
        let v_x = self.variance();
        let v_y = other.variance();

        let mut s = 0.0;
        for n in 0..self.len() {
            s += (self[n] - m_x) * (other[n] - m_y)
        }
        let c = 1.0 / self.len() as Dn * s / (v_x * v_y).sqrt();

        c
    }

    fn stddev(&self) -> Dn {
        self.variance().sqrt()
    }

    fn z_score(&self, check_value:Dn) -> Dn {
        (check_value - self.mean()) / self.stddev()
    }


    fn isolate_window_2d(&self, width_2d:usize, height_2d:usize, window_size:usize, x:usize, y:usize) -> DnVec {
        let mut v:DnVec= Vec::with_capacity(window_size * window_size);
        let start = window_size as i32 / 2 * -1;
        let end = window_size as i32 / 2 + 1;
        for _y in start..end as i32 {
            for _x in start..end as i32 {
                let get_x = x as i32 + _x;
                let get_y = y as i32 + _y;
                if get_x >= 0 
                    && get_x < width_2d as i32 
                    && get_y >= 0 
                    && get_y < height_2d as i32
                    {
                    v.push(self.get_2d(width_2d, height_2d, get_x as usize, get_y as usize));
                }
            }
        }
        v
    }

    fn get_2d(&self, width_2d:usize, height_2d:usize, x:usize, y:usize) -> Dn {
        if x >= width_2d || y >= height_2d {
            panic!("Invalid pixel coordinates");
        }
        let idx = y * width_2d + x;
        if idx >= self.len() {
            panic!("Index outside array bounds");
        }
        self[idx]
    }

    fn center_crop_2d(&self, from_width:usize, from_height:usize, to_width:usize, to_height:usize) -> DnVec {
        center_crop_2d(&self, from_width, from_height, to_width, to_height)
    }
    
    fn crop_2d(&self, from_width:usize, from_height:usize, left_x:usize, top_y:usize, to_width:usize, to_height:usize) -> DnVec {
        crop_2d(&self, from_width, from_height, left_x, top_y, to_width, to_height)
    }

    fn add(&self, other:&DnVec) -> DnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.add_mut(&other);
        n
    }

    fn add_mut(&mut self, other:&DnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).into_iter().for_each(|i|{
            self[i] += other[i];
        });
    }

    fn add_across(&self, other:Dn) -> DnVec {
        let mut n = self.clone();
        n.add_across_mut(other);
        n
    }

    fn add_across_mut(&mut self, other:Dn) {
        (0..self.len()).into_iter().for_each(|i|{
            self[i] = self[i] + other
        });
    }

    fn subtract(&self, other:&DnVec) -> DnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.subtract_mut(&other);
        n
    }

    fn subtract_mut(&mut self, other:&DnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).into_iter().for_each(|i|{
            self[i] -= other[i];
        });
    }

    fn subtract_across(&self, other:Dn) -> DnVec {
        let mut n = self.clone();
        n.subtract_across_mut(other);
        n
    }

    fn subtract_across_mut(&mut self, other:Dn) {
        (0..self.len()).into_iter().for_each(|i|{
            self[i] = self[i] - other
        });
    }

    fn divide(&self, other:&DnVec) -> DnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.divide_mut(&other);
        n
    }

    fn divide_mut(&mut self, other:&DnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).into_iter().for_each(|i|{
            self[i] /= other[i];
        });
    }


    fn divide_into(&self, divisor:Dn) -> DnVec {
        let mut n = self.clone();
        n.divide_into_mut(divisor);
        n
    }

    fn divide_into_mut(&mut self, divisor:Dn) {
        (0..self.len()).into_iter().for_each(|i|{
            self[i] = self[i] / divisor;
        });
    }


    fn scale(&self, scalar:Dn) -> DnVec {
        let mut n = self.clone();
        n.scale_mut(scalar);
        n
    }

    fn scale_mut(&mut self, scalar:Dn) {
        (0..self.len()).into_iter().for_each(|i|{
            self[i] = self[i] * scalar;
        });
    }


    fn multiply(&self, other:&DnVec) -> DnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.multiply_mut(&other);
        n
    }

    fn multiply_mut(&mut self, other:&DnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).into_iter().for_each(|i|{
            self[i] *= other[i];
        });
    }

    fn power(&self, exponent:Dn) -> DnVec {
        let mut n = self.clone();
        n.power_mut(exponent);
        n
    }

    fn power_mut(&mut self, exponent:Dn) {
        (0..self.len()).into_iter().for_each(|i|{
            self[i] = self[i].powf(exponent);
        });
    }

    fn clip(&self, clip_min:Dn, clip_max:Dn) -> DnVec {
        let mut n = self.clone();
        n.clip_mut(clip_min, clip_max);
        n
    }

    fn clip_mut(&mut self, clip_min:Dn, clip_max:Dn) {
        (0..self.len()).into_iter().for_each(|i|{
            self[i] = if self[i] > clip_max {
                clip_max
            } else if self[i] < clip_min {
                clip_min
            } else {
                self[i]
            };
        });
    }

    fn paste_2d(&self, dest_width:usize, dest_height:usize, src:&DnVec, src_width:usize, src_height:usize, tl_x:usize, tl_y:usize) -> DnVec {
        let mut n = self.clone();
        n.paste_mut_2d(dest_width, dest_height, src, src_width, src_height, tl_x, tl_y);
        n
    }

    fn paste_mut_2d(&mut self, dest_width:usize, dest_height:usize, src:&DnVec, src_width:usize, src_height:usize, tl_x:usize, tl_y:usize) {
        if dest_width * dest_height != self.len() {
            panic!("Invalid destination dimensions");
        }
        if src_width * src_height != src.len() {
            panic!("Invalid source dimensions");
        }
        if tl_x + src_width > dest_width {
            panic!("Source array too wide");
        }
        if tl_y + src_height > dest_height {
            panic!("Source array too high");
        }

        for y in 0..src_height {
            for x in 0..src_width {
                let dest_idx = (tl_y + y) * dest_width + (tl_x + x);
                let src_idx = (y * src_width) + x;
                self[dest_idx] = src[src_idx];
            }
        }
    }  

    fn normalize_force_minmax(&self, min:Dn, max:Dn, forced_min:Dn, forced_max:Dn) -> DnVec {
        let mut v = self.clone();
        v.normalize_force_minmax_mut(min, max, forced_min, forced_max);
        v
    }

    fn normalize_force_minmax_mut(&mut self, min:Dn, max:Dn, forced_min:Dn, forced_max:Dn) {
        for i in 0..self.len() {
            self[i] = ((self[i] - forced_min) / (forced_max- forced_min)) * (max - min) + min;
        }
    }
    
    fn get_min_max(&self) -> MinMax {
        let mut mm = MinMax{min: 0.0, max: 0.0};
        (0..self.len()).into_iter().for_each(|i|{
            mm.min = min!(mm.min, self[i]);
            mm.max = max!(mm.max, self[i]);
        });
        mm
    }

    fn normalize(&self, min:Dn, max:Dn) -> DnVec {
        let mut v = self.clone();
        v.normalize_mut(min, max);
        v
    }

    fn normalize_mut(&mut self, min:Dn, max:Dn) {
        let mm = self.get_min_max();
        self.normalize_force_minmax_mut(min, max, mm.min, mm.max);
    }

}