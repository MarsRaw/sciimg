use crate::{
    enums, image::Image, max, min, output, output::OutputFormat, path, Dn, DnVec, Mask, MaskVec,
    MaskedDnVec, MinMax, VecMath,
};

extern crate image;
use anyhow::Result;
use image::{open, DynamicImage, Rgba};
use itertools::iproduct;

// A simple image raster buffer.
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    pub buffer: MaskedDnVec,
    pub width: usize,
    pub height: usize,
    empty: bool,
    pub mode: enums::ImageMode,
}

#[derive(Debug, Clone)]
pub struct Offset {
    pub h: Dn,
    pub v: Dn,
}

#[allow(dead_code)]
impl ImageBuffer {
    // Creates a new image buffer of the requested width and height
    pub fn new(width: usize, height: usize) -> Result<ImageBuffer> {
        ImageBuffer::new_as_mode(width, height, enums::ImageMode::U16BIT)
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_as_mode(width: usize, height: usize, mode: enums::ImageMode) -> Result<ImageBuffer> {
        ImageBuffer::new_with_fill_as_mode(width, height, 0.0, mode)
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_with_fill(width: usize, height: usize, fill_value: f32) -> Result<ImageBuffer> {
        ImageBuffer::new_with_fill_as_mode(width, height, fill_value, enums::ImageMode::U16BIT)
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_with_fill_as_mode(
        width: usize,
        height: usize,
        fill_value: f32,
        mode: enums::ImageMode,
    ) -> Result<ImageBuffer> {
        Ok(ImageBuffer {
            buffer: MaskedDnVec::fill(width * height, fill_value),
            width,
            height,
            empty: false,
            mode,
        })
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_with_mask(width: usize, height: usize, mask: &MaskVec) -> Result<ImageBuffer> {
        Ok(ImageBuffer {
            buffer: MaskedDnVec::from_maskvec(mask),
            width,
            height,
            empty: false,
            mode: enums::ImageMode::U16BIT,
        })
    }

    pub fn new_with_mask_as(width: usize, height: usize, mask_value: bool) -> Result<ImageBuffer> {
        Ok(ImageBuffer {
            buffer: MaskedDnVec::fill_with_both(width * height, 0.0, mask_value),
            width,
            height,
            empty: false,
            mode: enums::ImageMode::U16BIT,
        })
    }

    fn new_with_mask_as_mode(
        width: usize,
        height: usize,
        mask: &MaskVec,
        mode: enums::ImageMode,
    ) -> Result<ImageBuffer> {
        Ok(ImageBuffer {
            buffer: MaskedDnVec::from_maskvec(mask),
            width,
            height,
            empty: false,
            mode,
        })
    }

    pub fn new_empty() -> Result<ImageBuffer> {
        Ok(ImageBuffer {
            buffer: MaskedDnVec::new(),
            width: 0,
            height: 0,
            empty: true,
            mode: enums::ImageMode::U16BIT,
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec(v: &DnVec, width: usize, height: usize) -> Result<ImageBuffer> {
        ImageBuffer::from_vec_as_mode(v, width, height, enums::ImageMode::U16BIT)
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_masked_vec(v: &MaskedDnVec, width: usize, height: usize) -> Result<ImageBuffer> {
        ImageBuffer::from_masked_vec_as_mode(v, width, height, enums::ImageMode::U16BIT)
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_as_mode(
        v: &DnVec,
        width: usize,
        height: usize,
        mode: enums::ImageMode,
    ) -> Result<ImageBuffer> {
        if v.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        Ok(ImageBuffer {
            buffer: MaskedDnVec::from_dnvec(v),
            width,
            height,
            empty: false,
            mode,
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_masked_vec_as_mode(
        v: &MaskedDnVec,
        width: usize,
        height: usize,
        mode: enums::ImageMode,
    ) -> Result<ImageBuffer> {
        if v.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        Ok(ImageBuffer {
            buffer: v.clone(),
            width,
            height,
            empty: false,
            mode,
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u8(v_u8: &Vec<u8>, width: usize, height: usize) -> Result<ImageBuffer> {
        if v_u8.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        let mut v = DnVec::zeros(width * height);
        for i in 0..v_u8.len() {
            v[i] = v_u8[i] as f32;
        }

        Ok(ImageBuffer {
            buffer: MaskedDnVec::from_dnvec(&v),
            width,
            height,
            empty: false,
            mode: enums::ImageMode::U16BIT,
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u8_with_mask(
        v_u8: &Vec<u8>,
        width: usize,
        height: usize,
        mask: &MaskVec,
    ) -> Result<ImageBuffer> {
        if v_u8.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        let mut v = DnVec::zeros(width * height);
        for i in 0..v_u8.len() {
            v[i] = v_u8[i] as f32;
        }

        Ok(ImageBuffer {
            buffer: MaskedDnVec::from_dnvec_and_mask(&v, mask),
            width,
            height,
            empty: false,
            mode: enums::ImageMode::U16BIT,
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u16(v_u16: &Vec<u16>, width: usize, height: usize) -> Result<ImageBuffer> {
        if v_u16.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        let mut v = DnVec::zeros(width * height);
        for i in 0..v_u16.len() {
            v[i] = v_u16[i] as f32;
        }

        Ok(ImageBuffer {
            buffer: MaskedDnVec::from_dnvec(&v),
            width,
            height,
            empty: false,
            mode: enums::ImageMode::U16BIT,
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u16_with_mask(
        v_u16: &Vec<u16>,
        width: usize,
        height: usize,
        mask: &MaskVec,
    ) -> Result<ImageBuffer> {
        if v_u16.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        let mut v = DnVec::zeros(width * height);
        for i in 0..v_u16.len() {
            v[i] = v_u16[i] as f32;
        }

        Ok(ImageBuffer {
            buffer: MaskedDnVec::from_dnvec_and_mask(&v, mask),
            width,
            height,
            empty: false,
            mode: enums::ImageMode::U16BIT,
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_with_mask(
        v: &DnVec,
        width: usize,
        height: usize,
        mask: &MaskVec,
    ) -> Result<ImageBuffer> {
        if v.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        Ok(ImageBuffer {
            buffer: MaskedDnVec::from_dnvec_and_mask(v, mask),
            width,
            height,
            empty: false,
            mode: enums::ImageMode::U16BIT,
        })
    }

    pub fn from_image_u8(
        image_data: &image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>,
    ) -> Result<ImageBuffer> {
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        let v = iproduct!(0..height, 0..width)
            .map(|(y, x)| image_data.get_pixel(x as u32, y as u32)[0] as f32)
            .collect();

        ImageBuffer::from_vec_as_mode(&v, width, height, enums::ImageMode::U16BIT)
    }

    pub fn from_image_u16(
        image_data: &image::ImageBuffer<image::Rgba<u16>, std::vec::Vec<u16>>,
    ) -> Result<ImageBuffer> {
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        let v = iproduct!(0..height, 0..width)
            .map(|(y, x)| image_data.get_pixel(x as u32, y as u32)[0] as f32)
            .collect();

        ImageBuffer::from_vec_as_mode(&v, width, height, enums::ImageMode::U16BIT)
    }

    pub fn from_file(file_path: &str) -> Result<ImageBuffer> {
        if !path::file_exists(file_path) {
            panic!("File not found: {}", file_path);
        }

        let image_data = open(file_path).unwrap().into_luma16();
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        let mut v = DnVec::zeros(width * height);

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let value = pixel[0] as f32;
                let idx = y * width + x;
                v[idx] = value;
            }
        }

        ImageBuffer::from_vec(&v, width, height)
    }

    fn new_from_op_masked(
        v: &MaskedDnVec,
        width: usize,
        height: usize,
        mode: enums::ImageMode,
    ) -> Result<ImageBuffer> {
        if v.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        Ok(ImageBuffer {
            buffer: v.clone(),
            width,
            height,
            empty: false,
            mode,
        })
    }

    fn new_from_op(
        v: &DnVec,
        width: usize,
        height: usize,
        mode: enums::ImageMode,
    ) -> Result<ImageBuffer> {
        if v.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        Ok(ImageBuffer {
            buffer: MaskedDnVec::from_dnvec(v),
            width,
            height,
            empty: false,
            mode,
        })
    }

    pub fn buffer_to_mask(buffer: &ImageBuffer) -> MaskVec {
        let mut m = MaskVec::new_mask(buffer.buffer.len());
        (0..buffer.buffer.len()).for_each(|i| {
            m[i] = buffer.buffer[i] > 0.0;
        });
        m
    }

    pub fn set_mask(&mut self, buffer: &ImageBuffer) {
        let mask = ImageBuffer::buffer_to_mask(buffer);
        self.buffer.apply_mask(&mask);
    }

    pub fn copy_mask_to(&self, dest: &mut ImageBuffer) {
        if self.width != dest.width || self.height != dest.height {
            panic!("Cannot copy into ImageBuffer: Incompatible dimensions");
        }
        (0..self.buffer.len()).for_each(|i| {
            dest.buffer.mask[i] = self.buffer.mask[i];
        });
    }

    pub fn clear_mask(&mut self) {
        self.buffer.clear_mask();
    }

    fn get_mask_at_index(&self, idx: usize) -> bool {
        self.buffer.mask_at(idx)
    }

    pub fn get_mask_at_point(&self, x: usize, y: usize) -> bool {
        let msk_idx = self.width * y + x;
        self.get_mask_at_index(msk_idx)
    }

    pub fn get_slice(&self, top_y: usize, len: usize) -> Result<ImageBuffer> {
        let start_index = top_y * self.width;
        let stop_index = (top_y + len) * self.width;

        let slice = self.buffer[start_index..stop_index].to_vec();

        ImageBuffer::from_vec(&slice, self.width, len)
    }

    pub fn to_vector_u8(&self) -> Vec<u8> {
        let need_len = self.buffer.len();
        let mut v: Vec<u8> = vec![0; need_len];

        (0..need_len).for_each(|i| {
            v[i] = match self.mode {
                enums::ImageMode::U8BIT => self.buffer[i] as u8,
                enums::ImageMode::U12BIT => (self.buffer[i] / 2033.0 * 255.0) as u8,
                enums::ImageMode::U16BIT => (self.buffer[i] / 65535.0 * 255.0) as u8,
            }
        });
        v
    }

    pub fn to_vector_u16(&self) -> Vec<u16> {
        let need_len = self.buffer.len();
        let mut v: Vec<u16> = vec![0; need_len];

        (0..need_len).for_each(|i| {
            v[i] = self.buffer[i] as u16;
        });
        v
    }

    pub fn to_vector(&self) -> Vec<f32> {
        self.buffer.to_vector()
    }

    pub fn to_masked_vector(&self) -> MaskedDnVec {
        self.buffer.clone()
    }

    pub fn to_mask(&self) -> MaskVec {
        self.buffer.mask.clone()
    }

    pub fn get_subframe(
        &self,
        left_x: usize,
        top_y: usize,
        width: usize,
        height: usize,
    ) -> Result<ImageBuffer> {
        let subframed_buffer =
            self.buffer
                .crop_2d(self.width, self.height, left_x, top_y, width, height);
        ImageBuffer::new_from_op_masked(&subframed_buffer, width, height, self.mode)
    }

    pub fn isolate_window(&self, window_size: usize, x: usize, y: usize) -> DnVec {
        self.buffer
            .isolate_window_2d(self.width, self.height, window_size, x, y)
            .to_vector()
    }

    #[inline(always)]
    pub fn get(&self, x: usize, y: usize) -> f32 {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.buffer[index]
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    pub fn get_interpolated(&self, x: f32, y: f32) -> Result<f32> {
        if x < self.width as f32 && y < self.height as f32 {
            let xf = x.floor();
            let xc = xf + 1.0;

            let yf = y.floor();
            let yc = yf + 1.0;

            let xd = x - xf;
            let yd = y - yf;

            let v00 = self.get(xf as usize, yf as usize);
            let v01 = self.get(xc as usize, yf as usize);
            let v10 = self.get(xf as usize, yc as usize);
            let v11 = self.get(xc as usize, yc as usize);

            let v0 = v10 * yd + v00 * (1.0 - yd);
            let v1 = v11 * yd + v01 * (1.0 - yd);
            let v = v1 * xd + v0 * (1.0 - xd);

            Ok(v)
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn put_u16(&mut self, x: usize, y: usize, val: u16) {
        self.put(x, y, val as f32);
    }

    pub fn put(&mut self, x: usize, y: usize, val: f32) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.buffer[index] = val;
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    pub fn put_mask(&mut self, x: usize, y: usize, m: bool) {
        if x < self.width && y < self.height {
            let index = y * self.width + x;
            self.buffer.mask[index] = m;
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    pub fn apply_lut_mut(&mut self, lut: &[u32]) {
        (0..self.buffer.len()).for_each(|i| {
            if self.buffer[i] < 0.0 || self.buffer[i] >= lut.len() as f32 {
                panic!(
                    "Buffer value {} is out of LUT lookup range: 0 - {}",
                    self.buffer[i],
                    lut.len()
                );
            }
            self.buffer[i] = lut[self.buffer[i] as usize] as f32;
        });
    }

    // Computes the mean of all pixel values
    pub fn mean(&self) -> Dn {
        self.buffer.mean()
    }

    pub fn sum(&self) -> Dn {
        self.buffer.sum()
    }

    pub fn divide_mut(&mut self, other: &ImageBuffer) {
        self.buffer.divide_mut(&other.buffer);
    }

    pub fn divide(&self, other: &ImageBuffer) -> Result<ImageBuffer> {
        ImageBuffer::new_from_op_masked(
            &self.buffer.divide(&other.buffer),
            self.width,
            self.height,
            self.mode,
        )
    }

    pub fn divide_into_mut(&mut self, divisor: Dn) {
        self.buffer.divide_into_mut(divisor);
    }

    pub fn divide_into(&self, divisor: Dn) -> Result<ImageBuffer> {
        ImageBuffer::new_from_op_masked(
            &self.buffer.divide_into(divisor),
            self.width,
            self.height,
            self.mode,
        )
    }

    pub fn scale_mut(&mut self, scalar: Dn) {
        self.buffer.scale_mut(scalar);
    }

    pub fn scale(&self, scalar: Dn) -> Result<ImageBuffer> {
        ImageBuffer::new_from_op_masked(
            &self.buffer.scale(scalar),
            self.width,
            self.height,
            self.mode,
        )
    }

    pub fn multiply_mut(&mut self, other: &ImageBuffer) {
        self.buffer.multiply_mut(&other.buffer);
    }

    pub fn multiply(&self, other: &ImageBuffer) -> Result<ImageBuffer> {
        ImageBuffer::new_from_op_masked(
            &self.buffer.multiply(&other.buffer),
            self.width,
            self.height,
            self.mode,
        )
    }

    pub fn add_mut(&mut self, other: &ImageBuffer) {
        self.buffer.add_mut(&other.buffer);
    }

    pub fn add_across(&self, other: Dn) -> Result<ImageBuffer> {
        let mut m = self.clone();
        m.add_across_mut(other);
        Ok(m)
    }

    pub fn add_across_mut(&mut self, other: Dn) {
        self.buffer.add_across_mut(other);
    }

    pub fn add(&self, other: &ImageBuffer) -> Result<ImageBuffer> {
        ImageBuffer::new_from_op_masked(
            &self.buffer.add(&other.buffer),
            self.width,
            self.height,
            self.mode,
        )
    }

    pub fn subtract_mut(&mut self, other: &ImageBuffer) {
        self.buffer.subtract_mut(&other.buffer);
    }

    pub fn subtract_across_mut(&mut self, other: Dn) {
        self.buffer.subtract_across_mut(other);
    }

    pub fn subtract(&self, other: &ImageBuffer) -> Result<ImageBuffer> {
        ImageBuffer::new_from_op_masked(
            &self.buffer.subtract(&other.buffer),
            self.width,
            self.height,
            self.mode,
        )
    }

    pub fn shift_to_min_zero(&self) -> Result<ImageBuffer> {
        let minmax = self.get_min_max();

        let mut v = self.buffer.clone();

        for i in 0..v.len() {
            let value = self.buffer[i];
            if minmax.min < 0.0 {
                v[i] = value + minmax.min;
            } else {
                v[i] = value - minmax.min;
            }
        }

        ImageBuffer::new_from_op_masked(&v, self.width, self.height, self.mode)
    }

    pub fn normalize_force_minmax_mut(
        &mut self,
        min: f32,
        max: f32,
        forced_min: f32,
        forced_max: f32,
    ) {
        self.buffer = self
            .buffer
            .normalize_force_minmax(min, max, forced_min, forced_max);
    }

    pub fn normalize_force_minmax(
        &self,
        min: f32,
        max: f32,
        forced_min: f32,
        forced_max: f32,
    ) -> Result<ImageBuffer> {
        ImageBuffer::new_from_op_masked(
            &self
                .buffer
                .normalize_force_minmax(min, max, forced_min, forced_max),
            self.width,
            self.height,
            self.mode,
        )
    }

    pub fn normalize(&self, min: f32, max: f32) -> Result<ImageBuffer> {
        let minmax = self.get_min_max();
        self.normalize_force_minmax(min, max, minmax.min, minmax.max)
    }

    pub fn normalize_mut(&mut self, min: f32, max: f32) {
        self.buffer = self.normalize(min, max).unwrap().buffer;
    }

    pub fn crop(&self, height: usize, width: usize) -> Result<ImageBuffer> {
        let cropped_buffer = self
            .buffer
            .center_crop_2d(self.width, self.height, width, height);
        ImageBuffer::new_from_op_masked(&cropped_buffer, width, height, self.mode)
    }

    pub fn clip(&self, clip_min: f32, clip_max: f32) -> Result<ImageBuffer> {
        ImageBuffer::new_from_op_masked(
            &self.buffer.clip(clip_min, clip_max),
            self.width,
            self.height,
            self.mode,
        )
    }

    pub fn clip_mut(&mut self, clip_min: f32, clip_max: f32) {
        self.buffer.clip_mut(clip_min, clip_max);
    }

    pub fn power(&self, power: f32) -> Result<ImageBuffer> {
        ImageBuffer::new_from_op_masked(
            &self.buffer.power(power),
            self.width,
            self.height,
            self.mode,
        )
    }

    pub fn power_mut(&mut self, power: f32) {
        self.buffer.power_mut(power);
    }

    pub fn calc_center_of_mass_offset(&self, threshold: Dn) -> Offset {
        let mut ox: Dn = 0.0;
        let mut oy: Dn = 0.0;
        let mut count: u32 = 0;

        for y in 0..self.height {
            for x in 0..self.width {
                let val = self.get(x, y);
                if val >= threshold {
                    ox += x as Dn;
                    oy += y as Dn;
                    count += 1;
                }
            }
        }

        if count > 0 {
            ox = (self.width as Dn / 2.0) - (ox / (count as Dn));
            oy = (self.height as Dn / 2.0) - (oy / (count as Dn));
        }

        Offset { h: ox, v: oy }
    }

    pub fn paste_mut(&mut self, src: &ImageBuffer, tl_x: usize, tl_y: usize) {
        self.buffer.paste_mut_2d(
            self.width,
            self.height,
            &src.buffer,
            src.width,
            src.height,
            tl_x,
            tl_y,
        );
    }

    pub fn paste(&self, src: &ImageBuffer, tl_x: usize, tl_y: usize) -> Result<ImageBuffer> {
        ImageBuffer::from_masked_vec(
            &self.buffer.paste_2d(
                self.width,
                self.height,
                &src.buffer,
                src.width,
                src.height,
                tl_x,
                tl_y,
            ),
            self.width,
            self.height,
        )
    }

    pub fn shift(&self, horiz: i32, vert: i32) -> Result<ImageBuffer> {
        let mut shifted_buffer =
            ImageBuffer::new_with_mask(self.width, self.height, &self.buffer.mask).unwrap();

        let h = self.height as i32;
        let w = self.width as i32;

        for y in 0..h {
            for x in 0..w {
                let shift_x = x + horiz;
                let shift_y = y + vert;

                if shift_x >= 0 && shift_y >= 0 && shift_x < w && shift_y < h {
                    shifted_buffer.put(
                        shift_x as usize,
                        shift_y as usize,
                        self.get(x as usize, y as usize),
                    );
                }
            }
        }
        Ok(shifted_buffer)
    }

    pub fn shift_interpolated(&self, horiz: f32, vert: f32) -> Result<ImageBuffer> {
        let mut shifted_buffer =
            ImageBuffer::new_with_mask(self.width, self.height, &self.buffer.mask).unwrap();

        let h = self.height as i32 - 1;
        let w = self.width as i32 - 1;

        let hf = horiz - horiz.floor();
        let vf = vert - vert.floor();

        for y in 0..h {
            for x in 0..w {
                let shift_x = x as f32 + horiz.floor();
                let shift_y = y as f32 + vert.floor();

                if shift_x >= 0.0 && shift_y >= 0.0 && shift_x < w as f32 && shift_y < h as f32 {
                    shifted_buffer.put(
                        shift_x as usize,
                        shift_y as usize,
                        self.get_interpolated(x as f32 - (1.0 - hf), y as f32 - (1.0 - vf))
                            .unwrap(),
                    );
                }
            }
        }
        Ok(shifted_buffer)
    }

    pub fn get_min_max(&self) -> MinMax {
        self.buffer.get_min_max()
    }

    pub fn get_min_max_ignore_black(&self) -> MinMax {
        let mut mm = MinMax {
            min: std::f32::MAX,
            max: std::f32::MIN,
        };
        (0..self.buffer.len()).for_each(|i| {
            if self.buffer[i] != std::f32::INFINITY && self.buffer[i] > 0.0 {
                mm.min = min!(mm.min, self.buffer[i]);
                mm.max = max!(mm.max, self.buffer[i]);
            }
        });
        mm
    }

    pub fn buffer_to_image_8bit(&self) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
        let mut out_img =
            DynamicImage::new_rgba8(self.width as u32, self.height as u32).into_rgba8();

        iproduct!(0..self.height, 0..self.width).for_each(|(y, x)| {
            let val = self.get(x, y).round() as u8;
            let a = if self.get_mask_at_point(x, y) {
                std::u8::MAX
            } else {
                std::u8::MIN
            };
            out_img.put_pixel(x as u32, y as u32, Rgba([val, val, val, a]));
        });

        out_img
    }

    pub fn buffer_to_image_16bit(
        &self,
    ) -> image::ImageBuffer<image::Rgba<u16>, std::vec::Vec<u16>> {
        let mut out_img =
            DynamicImage::new_rgba16(self.width as u32, self.height as u32).into_rgba16();

        iproduct!(0..self.height, 0..self.width).for_each(|(y, x)| {
            let val = self.get(x, y).round() as u16;
            let a = if self.get_mask_at_point(x, y) {
                std::u16::MAX
            } else {
                std::u16::MIN
            };
            out_img.put_pixel(x as u32, y as u32, Rgba([val, val, val, a]));
        });

        out_img
    }

    pub fn save_use_mode(&self, to_file: &str, mode: enums::ImageMode) -> Result<()> {
        match output::get_default_output_format() {
            Ok(format) => self.save_with_mode_and_format(to_file, mode, format),
            Err(why) => Err(why),
        }
    }

    pub fn save(&self, to_file: &str) -> Result<()> {
        self.save_use_mode(to_file, self.mode)
    }

    pub fn save_with_mode_and_format(
        &self,
        to_file: &str,
        mode: enums::ImageMode,
        format: OutputFormat,
    ) -> Result<()> {
        output::save_image_with_format(
            to_file,
            format,
            &Image::new_from_buffer_mono_use_mode(self, mode).unwrap(),
        )
    }
}
