use std::ops::{Index, IndexMut};

pub mod blur;
pub mod camera;
pub mod debayer;
pub mod decompanding;
pub mod enums;
pub mod error;
pub mod hotpixel;
pub mod imagebuffer;
pub mod imagerot;
pub mod inpaint;
pub mod lowpass;
pub mod matrix;
pub mod noise;
pub mod path;
pub mod prelude;
pub mod quality;
pub mod quaternion;
pub mod resize;
pub mod rgbimage;
pub mod stats;
pub mod util;
pub mod vector;

// Dn -> Digital number / image pixel value as 32 bit floating point.
pub type Dn = f32;
pub type DnVec = Vec<Dn>;

pub fn center_crop_2d<T: Copy>(
    from_array: &[T],
    from_width: usize,
    from_height: usize,
    to_width: usize,
    to_height: usize,
) -> Vec<T> {
    let mut new_arr: Vec<T> = Vec::with_capacity(to_width * to_height);

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

pub fn crop_2d<T: Copy>(
    from_array: &[T],
    from_width: usize,
    from_height: usize,
    left_x: usize,
    top_y: usize,
    to_width: usize,
    to_height: usize,
) -> Vec<T> {
    if top_y + to_height > from_height || left_x + to_width > from_width {
        panic!("Crop bounds exceeed source array");
    }

    let mut new_arr: Vec<T> = Vec::with_capacity(to_width * to_height);

    for y in 0..to_height {
        for x in 0..to_width {
            let from_idx = (top_y + y) * from_width + (left_x + x);
            new_arr.push(from_array[from_idx]);
        }
    }
    new_arr
}

pub fn isolate_window_2d<T: Copy>(
    from_array: &[T],
    width_2d: usize,
    height_2d: usize,
    window_size: usize,
    x: usize,
    y: usize,
) -> Vec<T> {
    let mut v: Vec<T> = Vec::with_capacity(window_size * window_size);
    let start = -(window_size as i32 / 2);
    let end = window_size as i32 / 2 + 1;
    for _y in start..end {
        for _x in start..end {
            let get_x = x as i32 + _x;
            let get_y = y as i32 + _y;
            if get_x >= 0 && get_x < width_2d as i32 && get_y >= 0 && get_y < height_2d as i32 {
                let idx = get_y * width_2d as i32 + get_x;
                v.push(from_array[idx as usize]);
            }
        }
    }
    v
}

#[derive(Debug, Clone)]
pub struct MinMax {
    pub min: Dn,
    pub max: Dn,
}

//////////////////////////////////////////////////
/// Dn Vector (Unmasked)
//////////////////////////////////////////////////

pub trait VecMath {
    fn fill(capacity: usize, fill_value: Dn) -> Self;
    fn zeros(capacity: usize) -> Self;
    fn sum(&self) -> Dn;
    fn mean(&self) -> Dn;
    fn variance(&self) -> Dn;
    fn xcorr(&self, other: &Self) -> Dn;
    fn stddev(&self) -> Dn;
    fn z_score(&self, check_value: Dn) -> Dn;
    fn isolate_window_2d(
        &self,
        width_2d: usize,
        height_2d: usize,
        window_size: usize,
        x: usize,
        y: usize,
    ) -> Self;
    fn get_2d(&self, width_2d: usize, height_2d: usize, x: usize, y: usize) -> Dn;
    fn center_crop_2d(
        &self,
        from_width: usize,
        from_height: usize,
        to_width: usize,
        to_height: usize,
    ) -> Self;
    fn crop_2d(
        &self,
        from_width: usize,
        from_height: usize,
        left_x: usize,
        top_y: usize,
        to_width: usize,
        to_height: usize,
    ) -> Self;

    fn add(&self, other: &Self) -> Self;
    fn add_mut(&mut self, other: &Self);

    fn add_across(&self, other: Dn) -> Self;
    fn add_across_mut(&mut self, other: Dn);

    fn subtract(&self, other: &Self) -> Self;
    fn subtract_mut(&mut self, other: &Self);

    fn subtract_across(&self, other: Dn) -> Self;
    fn subtract_across_mut(&mut self, other: Dn);

    fn divide(&self, other: &Self) -> Self;
    fn divide_mut(&mut self, other: &Self);

    fn divide_into(&self, divisor: Dn) -> Self;
    fn divide_into_mut(&mut self, divisor: Dn);

    fn scale(&self, scalar: Dn) -> Self;
    fn scale_mut(&mut self, scalar: Dn);

    fn multiply(&self, other: &Self) -> Self;
    fn multiply_mut(&mut self, other: &Self);

    fn power(&self, exponent: Dn) -> Self;
    fn power_mut(&mut self, exponent: Dn);

    fn clip(&self, clip_min: Dn, clip_max: Dn) -> Self;
    fn clip_mut(&mut self, clip_min: Dn, clip_max: Dn);

    #[allow(clippy::too_many_arguments)]
    fn paste_2d(
        &self,
        dest_width: usize,
        dest_height: usize,
        src: &Self,
        src_width: usize,
        src_height: usize,
        tl_x: usize,
        tl_y: usize,
    ) -> Self;

    #[allow(clippy::too_many_arguments)]
    fn paste_mut_2d(
        &mut self,
        dest_width: usize,
        dest_height: usize,
        src: &Self,
        src_width: usize,
        src_height: usize,
        tl_x: usize,
        tl_y: usize,
    );

    fn normalize_force_minmax(&self, min: Dn, max: Dn, forced_min: Dn, forced_max: Dn) -> Self;
    fn normalize_force_minmax_mut(&mut self, min: Dn, max: Dn, forced_min: Dn, forced_max: Dn);

    fn min(&self) -> Dn;
    fn max(&self) -> Dn;
    fn get_min_max(&self) -> MinMax;

    fn normalize(&self, min: Dn, max: Dn) -> Self;
    fn normalize_mut(&mut self, min: Dn, max: Dn);
}

impl VecMath for DnVec {
    fn fill(capacity: usize, fill_value: Dn) -> DnVec {
        let mut v: DnVec = Vec::with_capacity(capacity);
        v.resize(capacity, fill_value);
        v
    }

    fn zeros(capacity: usize) -> DnVec {
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

    fn xcorr(&self, other: &DnVec) -> Dn {
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
        1.0 / self.len() as Dn * s / (v_x * v_y).sqrt()
    }

    fn stddev(&self) -> Dn {
        self.variance().sqrt()
    }

    fn z_score(&self, check_value: Dn) -> Dn {
        (check_value - self.mean()) / self.stddev()
    }

    fn isolate_window_2d(
        &self,
        width_2d: usize,
        height_2d: usize,
        window_size: usize,
        x: usize,
        y: usize,
    ) -> DnVec {
        isolate_window_2d(self, width_2d, height_2d, window_size, x, y)
    }

    fn get_2d(&self, width_2d: usize, height_2d: usize, x: usize, y: usize) -> Dn {
        if x >= width_2d || y >= height_2d {
            panic!("Invalid pixel coordinates");
        }
        let idx = y * width_2d + x;
        if idx >= self.len() {
            panic!("Index outside array bounds");
        }
        self[idx]
    }

    fn center_crop_2d(
        &self,
        from_width: usize,
        from_height: usize,
        to_width: usize,
        to_height: usize,
    ) -> DnVec {
        center_crop_2d(self, from_width, from_height, to_width, to_height)
    }

    fn crop_2d(
        &self,
        from_width: usize,
        from_height: usize,
        left_x: usize,
        top_y: usize,
        to_width: usize,
        to_height: usize,
    ) -> DnVec {
        crop_2d(
            self,
            from_width,
            from_height,
            left_x,
            top_y,
            to_width,
            to_height,
        )
    }

    fn add(&self, other: &DnVec) -> DnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.add_mut(other);
        n
    }

    fn add_mut(&mut self, other: &DnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).for_each(|i| {
            self[i] += other[i];
        });
    }

    fn add_across(&self, other: Dn) -> DnVec {
        let mut n = self.clone();
        n.add_across_mut(other);
        n
    }

    fn add_across_mut(&mut self, other: Dn) {
        (0..self.len()).for_each(|i| self[i] += other);
    }

    fn subtract(&self, other: &DnVec) -> DnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.subtract_mut(other);
        n
    }

    fn subtract_mut(&mut self, other: &DnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).for_each(|i| {
            self[i] -= other[i];
        });
    }

    fn subtract_across(&self, other: Dn) -> DnVec {
        let mut n = self.clone();
        n.subtract_across_mut(other);
        n
    }

    fn subtract_across_mut(&mut self, other: Dn) {
        (0..self.len()).for_each(|i| self[i] -= other);
    }

    fn divide(&self, other: &DnVec) -> DnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.divide_mut(other);
        n
    }

    fn divide_mut(&mut self, other: &DnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).for_each(|i| {
            self[i] /= other[i];
        });
    }

    fn divide_into(&self, divisor: Dn) -> DnVec {
        let mut n = self.clone();
        n.divide_into_mut(divisor);
        n
    }

    fn divide_into_mut(&mut self, divisor: Dn) {
        (0..self.len()).for_each(|i| {
            self[i] /= divisor;
        });
    }

    fn scale(&self, scalar: Dn) -> DnVec {
        let mut n = self.clone();
        n.scale_mut(scalar);
        n
    }

    fn scale_mut(&mut self, scalar: Dn) {
        (0..self.len()).for_each(|i| {
            self[i] *= scalar;
        });
    }

    fn multiply(&self, other: &DnVec) -> DnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.multiply_mut(other);
        n
    }

    fn multiply_mut(&mut self, other: &DnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).for_each(|i| {
            self[i] *= other[i];
        });
    }

    fn power(&self, exponent: Dn) -> DnVec {
        let mut n = self.clone();
        n.power_mut(exponent);
        n
    }

    fn power_mut(&mut self, exponent: Dn) {
        (0..self.len()).for_each(|i| {
            self[i] = self[i].powf(exponent);
        });
    }

    fn clip(&self, clip_min: Dn, clip_max: Dn) -> DnVec {
        let mut n = self.clone();
        n.clip_mut(clip_min, clip_max);
        n
    }

    fn clip_mut(&mut self, clip_min: Dn, clip_max: Dn) {
        (0..self.len()).for_each(|i| {
            self[i] = if self[i] > clip_max {
                clip_max
            } else if self[i] < clip_min {
                clip_min
            } else {
                self[i]
            };
        });
    }

    fn paste_2d(
        &self,
        dest_width: usize,
        dest_height: usize,
        src: &DnVec,
        src_width: usize,
        src_height: usize,
        tl_x: usize,
        tl_y: usize,
    ) -> DnVec {
        let mut n = self.clone();
        n.paste_mut_2d(
            dest_width,
            dest_height,
            src,
            src_width,
            src_height,
            tl_x,
            tl_y,
        );
        n
    }

    fn paste_mut_2d(
        &mut self,
        dest_width: usize,
        dest_height: usize,
        src: &DnVec,
        src_width: usize,
        src_height: usize,
        tl_x: usize,
        tl_y: usize,
    ) {
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

    fn normalize_force_minmax(&self, min: Dn, max: Dn, forced_min: Dn, forced_max: Dn) -> DnVec {
        let mut v = self.clone();
        v.normalize_force_minmax_mut(min, max, forced_min, forced_max);
        v
    }

    fn normalize_force_minmax_mut(&mut self, min: Dn, max: Dn, forced_min: Dn, forced_max: Dn) {
        (0..self.len()).for_each(|i| {
            self[i] = ((self[i] - forced_min) / (forced_max - forced_min)) * (max - min) + min;
        });
    }

    fn min(&self) -> Dn {
        let mut m = std::f32::MAX;
        (0..self.len()).for_each(|i| {
            m = min!(m, self[i]);
        });
        m
    }

    fn max(&self) -> Dn {
        let mut m = std::f32::MIN;
        (0..self.len()).for_each(|i| {
            m = max!(m, self[i]);
        });
        m
    }

    fn get_min_max(&self) -> MinMax {
        let mut mm = MinMax {
            min: std::f32::MAX,
            max: std::f32::MIN,
        };
        (0..self.len()).for_each(|i| {
            mm.min = min!(mm.min, self[i]);
            mm.max = max!(mm.max, self[i]);
        });
        mm
    }

    fn normalize(&self, min: Dn, max: Dn) -> DnVec {
        let mut v = self.clone();
        v.normalize_mut(min, max);
        v
    }

    fn normalize_mut(&mut self, min: Dn, max: Dn) {
        let mm = self.get_min_max();
        self.normalize_force_minmax_mut(min, max, mm.min, mm.max);
    }
}

//////////////////////////////////////////////////
/// Mask
//////////////////////////////////////////////////

pub type MaskVec = Vec<bool>;
pub trait Mask {
    fn new_mask(capacity: usize) -> MaskVec;
    fn fill_mask(capacity: usize, fill_value: bool) -> MaskVec;
    fn get_2d(&self, width_2d: usize, height_2d: usize, x: usize, y: usize) -> bool;
    fn put_2d(&mut self, width_2d: usize, height_2d: usize, x: usize, y: usize, value: bool);
    fn clear_mask(&mut self);
}

impl Mask for MaskVec {
    fn new_mask(capacity: usize) -> MaskVec {
        MaskVec::fill_mask(capacity, true)
    }

    fn fill_mask(capacity: usize, fill_value: bool) -> MaskVec {
        let mut v: MaskVec = Vec::with_capacity(capacity);
        v.resize(capacity, fill_value);
        v
    }

    fn get_2d(&self, width_2d: usize, height_2d: usize, x: usize, y: usize) -> bool {
        if x >= width_2d || y >= height_2d {
            panic!("Invalid pixel coordinates");
        }
        let idx = y * width_2d + x;
        if idx >= self.len() {
            panic!("Index outside array bounds");
        }
        self[idx]
    }

    fn put_2d(&mut self, width_2d: usize, height_2d: usize, x: usize, y: usize, value: bool) {
        if x >= width_2d || y >= height_2d {
            panic!("Invalid pixel coordinates");
        }
        let idx = y * width_2d + x;
        if idx >= self.len() {
            panic!("Index outside array bounds");
        }
        self[idx] = value;
    }

    fn clear_mask(&mut self) {
        (0..self.len()).for_each(|i| {
            self[i] = true;
        });
    }
}

//////////////////////////////////////////////////
/// Dn Vector (Masked)
//////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct MaskedDnVec {
    vec: DnVec,
    pub mask: MaskVec,
    null: Dn, // The same idea as /dev/null
}

pub struct MaskedDnVecIter<'a> {
    vec: &'a MaskedDnVec,
    curr: usize,
    next: usize,
}

impl MaskedDnVecIter<'_> {
    pub fn new(vec: &MaskedDnVec) -> MaskedDnVecIter<'_> {
        MaskedDnVecIter {
            vec,
            curr: 0,
            next: 1,
        }
    }
}

impl Iterator for MaskedDnVecIter<'_> {
    type Item = Dn;

    fn next(&mut self) -> Option<Self::Item> {
        let v = if self.curr >= self.vec.len() {
            None
        } else {
            Some(self.vec[self.curr])
        };

        let new_next = self.next + 1;
        self.curr = self.next;
        self.next = new_next;

        v
    }
}

impl Default for MaskedDnVec {
    fn default() -> Self {
        Self::new()
    }
}

impl MaskedDnVec {
    pub fn new() -> Self {
        MaskedDnVec {
            vec: DnVec::new(),
            mask: MaskVec::new(),
            null: 0.0,
        }
    }
    pub fn from_dnvec(vec: &DnVec) -> Self {
        MaskedDnVec {
            vec: vec.clone(),
            mask: MaskVec::new_mask(vec.len()),
            null: 0.0,
        }
    }

    pub fn from_maskvec(vec: &MaskVec) -> Self {
        MaskedDnVec {
            vec: DnVec::zeros(vec.len()),
            mask: vec.clone(),
            null: 0.0,
        }
    }

    pub fn from_dnvec_and_mask(vec: &DnVec, mask: &MaskVec) -> Self {
        if vec.len() != mask.len() {
            panic!("DnVec and MaskVec are different lengths");
        }
        MaskedDnVec {
            vec: vec.clone(),
            mask: mask.clone(),
            null: 0.0,
        }
    }

    pub fn to_vector(&self) -> DnVec {
        let mut v = self.vec.clone();
        (0..v.len()).for_each(|i| {
            if self.mask_at(i) {
                v[i] = self.vec[i];
            } else {
                v[i] = 0.0;
            }
        });
        v
    }

    pub fn apply_mask(&mut self, mask: &MaskVec) {
        if self.mask.len() != mask.len() {
            panic!("DnVec and MaskVec are different lengths");
        }
        (0..self.mask.len()).for_each(|i| {
            self.mask[i] = mask[i];
        });
    }

    pub fn mask_at(&self, index: usize) -> bool {
        self.mask[index]
    }

    pub fn clear_mask(&mut self) {
        (0..self.mask.len()).for_each(|i| {
            self.mask[i] = true;
        });
    }

    pub fn fill_with_both(capacity: usize, dn_fill_value: Dn, mask_fill_value: bool) -> Self {
        MaskedDnVec {
            vec: DnVec::fill(capacity, dn_fill_value),
            mask: MaskVec::fill_mask(capacity, mask_fill_value),
            null: 0.0,
        }
    }

    pub fn fill_with_mask(capacity: usize, fill_value: Dn, mask: &MaskVec) -> Self {
        if mask.len() != capacity {
            panic!("Mask length does not equal vector capacity");
        }
        MaskedDnVec {
            vec: DnVec::fill(capacity, fill_value),
            mask: mask.clone(),
            null: 0.0,
        }
    }

    pub fn zeros_with_mask(capacity: usize, mask: &MaskVec) -> Self {
        if mask.len() != capacity {
            panic!("Mask length does not equal vector capacity");
        }
        MaskedDnVec {
            vec: DnVec::zeros(capacity),
            mask: mask.clone(),
            null: 0.0,
        }
    }

    pub fn len(&self) -> usize {
        self.vec.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn iter(&self) -> MaskedDnVecIter<'_> {
        MaskedDnVecIter::new(self)
    }
}

impl Index<usize> for MaskedDnVec {
    type Output = Dn;
    fn index<'a>(&'_ self, i: usize) -> &'_ Dn {
        if self.mask[i] {
            &self.vec[i]
        } else {
            &0.0
        }
    }
}

impl Index<std::ops::Range<usize>> for MaskedDnVec {
    type Output = [Dn];
    fn index<'a>(&'_ self, i: std::ops::Range<usize>) -> &'_ [Dn] {
        &self.vec[i.start..i.end]
        // TODO: Actually mask the stuff
    }
}

impl IndexMut<usize> for MaskedDnVec {
    fn index_mut<'a>(&'_ mut self, i: usize) -> &'_ mut Dn {
        if self.mask[i] {
            &mut self.vec[i]
        } else {
            &mut self.null
        }
    }
}

impl IndexMut<std::ops::Range<usize>> for MaskedDnVec {
    fn index_mut<'a>(&'_ mut self, i: std::ops::Range<usize>) -> &'_ mut [Dn] {
        &mut self.vec[i.start..i.end]
        // TODO: Actually mask the stuff
    }
}

impl VecMath for MaskedDnVec {
    fn fill(capacity: usize, fill_value: Dn) -> MaskedDnVec {
        let mask = MaskVec::new_mask(capacity);
        MaskedDnVec {
            vec: DnVec::fill(capacity, fill_value),
            mask,
            null: 0.0,
        }
    }

    fn zeros(capacity: usize) -> MaskedDnVec {
        MaskedDnVec::fill(capacity, 0.0)
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

    fn xcorr(&self, other: &MaskedDnVec) -> Dn {
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
        1.0 / self.len() as Dn * s / (v_x * v_y).sqrt()
    }

    fn stddev(&self) -> Dn {
        self.variance().sqrt()
    }

    fn z_score(&self, check_value: Dn) -> Dn {
        (check_value - self.mean()) / self.stddev()
    }

    fn isolate_window_2d(
        &self,
        width_2d: usize,
        height_2d: usize,
        window_size: usize,
        x: usize,
        y: usize,
    ) -> MaskedDnVec {
        let isolated_vec = isolate_window_2d(&self.vec, width_2d, height_2d, window_size, x, y);
        let isolated_mask = isolate_window_2d(&self.mask, width_2d, height_2d, window_size, x, y);
        MaskedDnVec {
            vec: isolated_vec,
            mask: isolated_mask,
            null: 0.0,
        }
    }

    fn get_2d(&self, width_2d: usize, height_2d: usize, x: usize, y: usize) -> Dn {
        if x >= width_2d || y >= height_2d {
            panic!("Invalid pixel coordinates");
        }
        let idx = y * width_2d + x;
        if idx >= self.len() {
            panic!("Index outside array bounds");
        }
        self[idx]
    }

    fn center_crop_2d(
        &self,
        from_width: usize,
        from_height: usize,
        to_width: usize,
        to_height: usize,
    ) -> MaskedDnVec {
        let cropped_vec = center_crop_2d(&self.vec, from_width, from_height, to_width, to_height);
        let cropped_mask = center_crop_2d(&self.mask, from_width, from_height, to_width, to_height);
        MaskedDnVec {
            vec: cropped_vec,
            mask: cropped_mask,
            null: 0.0,
        }
    }

    fn crop_2d(
        &self,
        from_width: usize,
        from_height: usize,
        left_x: usize,
        top_y: usize,
        to_width: usize,
        to_height: usize,
    ) -> MaskedDnVec {
        let cropped_vec = crop_2d(
            &self.vec,
            from_width,
            from_height,
            left_x,
            top_y,
            to_width,
            to_height,
        );
        let cropped_mask = crop_2d(
            &self.mask,
            from_width,
            from_height,
            left_x,
            top_y,
            to_width,
            to_height,
        );
        MaskedDnVec {
            vec: cropped_vec,
            mask: cropped_mask,
            null: 0.0,
        }
    }

    fn add(&self, other: &MaskedDnVec) -> MaskedDnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.add_mut(other);
        n
    }

    fn add_mut(&mut self, other: &MaskedDnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).for_each(|i| {
            self[i] += other[i];
        });
    }

    fn add_across(&self, other: Dn) -> MaskedDnVec {
        let mut n = self.clone();
        n.add_across_mut(other);
        n
    }

    fn add_across_mut(&mut self, other: Dn) {
        (0..self.len()).for_each(|i| self[i] += other);
    }

    fn subtract(&self, other: &MaskedDnVec) -> MaskedDnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.subtract_mut(other);
        n
    }

    fn subtract_mut(&mut self, other: &MaskedDnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).for_each(|i| {
            self[i] -= other[i];
        });
    }

    fn subtract_across(&self, other: Dn) -> MaskedDnVec {
        let mut n = self.clone();
        n.subtract_across_mut(other);
        n
    }

    fn subtract_across_mut(&mut self, other: Dn) {
        (0..self.len()).for_each(|i| self[i] -= other);
    }

    fn divide(&self, other: &MaskedDnVec) -> MaskedDnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.divide_mut(other);
        n
    }

    fn divide_mut(&mut self, other: &MaskedDnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).for_each(|i| {
            self[i] /= other[i];
        });
    }

    fn divide_into(&self, divisor: Dn) -> MaskedDnVec {
        let mut n = self.clone();
        n.divide_into_mut(divisor);
        n
    }

    fn divide_into_mut(&mut self, divisor: Dn) {
        (0..self.len()).for_each(|i| {
            self[i] /= divisor;
        });
    }

    fn scale(&self, scalar: Dn) -> MaskedDnVec {
        let mut n = self.clone();
        n.scale_mut(scalar);
        n
    }

    fn scale_mut(&mut self, scalar: Dn) {
        (0..self.len()).for_each(|i| {
            self[i] *= scalar;
        });
    }

    fn multiply(&self, other: &MaskedDnVec) -> MaskedDnVec {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        let mut n = self.clone();
        n.multiply_mut(other);
        n
    }

    fn multiply_mut(&mut self, other: &MaskedDnVec) {
        if self.len() != other.len() {
            panic!("Array size mismatch");
        }

        (0..self.len()).for_each(|i| {
            self[i] *= other[i];
        });
    }

    fn power(&self, exponent: Dn) -> MaskedDnVec {
        let mut n = self.clone();
        n.power_mut(exponent);
        n
    }

    fn power_mut(&mut self, exponent: Dn) {
        (0..self.len()).for_each(|i| {
            self[i] = self[i].powf(exponent);
        });
    }

    fn clip(&self, clip_min: Dn, clip_max: Dn) -> MaskedDnVec {
        let mut n = self.clone();
        n.clip_mut(clip_min, clip_max);
        n
    }

    fn clip_mut(&mut self, clip_min: Dn, clip_max: Dn) {
        (0..self.len()).for_each(|i| {
            self[i] = if self[i] > clip_max {
                clip_max
            } else if self[i] < clip_min {
                clip_min
            } else {
                self[i]
            };
        });
    }

    fn paste_2d(
        &self,
        dest_width: usize,
        dest_height: usize,
        src: &MaskedDnVec,
        src_width: usize,
        src_height: usize,
        tl_x: usize,
        tl_y: usize,
    ) -> MaskedDnVec {
        let mut n = self.clone();
        n.paste_mut_2d(
            dest_width,
            dest_height,
            src,
            src_width,
            src_height,
            tl_x,
            tl_y,
        );
        n
    }

    fn paste_mut_2d(
        &mut self,
        dest_width: usize,
        dest_height: usize,
        src: &MaskedDnVec,
        src_width: usize,
        src_height: usize,
        tl_x: usize,
        tl_y: usize,
    ) {
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

    fn normalize_force_minmax(
        &self,
        min: Dn,
        max: Dn,
        forced_min: Dn,
        forced_max: Dn,
    ) -> MaskedDnVec {
        let mut v = self.clone();
        v.normalize_force_minmax_mut(min, max, forced_min, forced_max);
        v
    }

    fn normalize_force_minmax_mut(&mut self, min: Dn, max: Dn, forced_min: Dn, forced_max: Dn) {
        for i in 0..self.len() {
            self[i] = ((self[i] - forced_min) / (forced_max - forced_min)) * (max - min) + min;
        }
    }

    fn min(&self) -> Dn {
        let mut m = std::f32::MAX;
        (0..self.len()).for_each(|i| {
            m = min!(m, self[i]);
        });
        m
    }

    fn max(&self) -> Dn {
        let mut m = std::f32::MIN;
        (0..self.len()).for_each(|i| {
            m = max!(m, self[i]);
        });
        m
    }

    fn get_min_max(&self) -> MinMax {
        let mut mm = MinMax {
            min: std::f32::MAX,
            max: std::f32::MIN,
        };
        (0..self.len()).for_each(|i| {
            if self[i] != std::f32::INFINITY {
                mm.min = min!(mm.min, self[i]);
                mm.max = max!(mm.max, self[i]);
            }
        });
        mm
    }

    fn normalize(&self, min: Dn, max: Dn) -> MaskedDnVec {
        let mut v = self.clone();
        v.normalize_mut(min, max);
        v
    }

    fn normalize_mut(&mut self, min: Dn, max: Dn) {
        let mm = self.get_min_max();
        self.normalize_force_minmax_mut(min, max, mm.min, mm.max);
    }
}
