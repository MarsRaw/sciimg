
use crate::{
    error, 
    enums,
    path
};

extern crate image;
use image::{
    open, 
    DynamicImage, 
    Rgba
};

// A simple image raster buffer.
#[derive(Debug, Clone)]
pub struct ImageBuffer {
    pub buffer: Vec<f32>,
    pub width: usize,
    pub height: usize,
    empty: bool,
    pub mask: Option<Vec<bool>>,
    pub mode: enums::ImageMode
}

#[derive(Debug, Clone)]
pub struct Offset {
    pub h: f32,
    pub v: f32,
}

#[derive(Debug, Clone)]
pub struct MinMax {
    pub min: f32,
    pub max: f32,
}

// Implements a center crop
fn crop_array<T:Copy>(arr:&[T], from_width:usize, from_height:usize, to_width:usize, to_height:usize) -> Vec<T> {
    let mut new_arr : Vec<T> = Vec::with_capacity(to_width * to_height);
 
    for y in 0..to_height {
        for x in 0..to_width {
    
            let from_x = ((from_width - to_width) / 2) + x;
            let from_y = ((from_height - to_height) / 2) + y;
            let from_idx = from_y * from_width + from_x;

            //let to_idx = y * to_width + x;
            new_arr.push(arr[from_idx]);
        }
    }
    
    new_arr
}

fn subframe_array<T:Copy>(arr:&[T], from_width:usize, _from_height:usize, left_x:usize, top_y:usize, to_width:usize, to_height:usize) -> Vec<T> {
    let mut new_arr : Vec<T> = Vec::with_capacity(to_width * to_height);

    for y in 0..to_height {
        for x in 0..to_width {
            let from_idx = (top_y + y) * from_width + (left_x + x);
            new_arr.push(arr[from_idx]);
        }
    }
    new_arr
}


#[allow(dead_code)]
impl ImageBuffer {


    // Creates a new image buffer of the requested width and height
    pub fn new(width:usize, height:usize) -> error::Result<ImageBuffer> {
        ImageBuffer::new_as_mode(width, height, enums::ImageMode::U16BIT)
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_as_mode(width:usize, height:usize, mode:enums::ImageMode) -> error::Result<ImageBuffer> {
        ImageBuffer::new_with_fill_as_mode(width, height, 0.0, mode)
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_with_fill(width:usize, height:usize, fill_value:f32) -> error::Result<ImageBuffer> {
        ImageBuffer::new_with_fill_as_mode(width, height, fill_value, enums::ImageMode::U16BIT)
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_with_fill_as_mode(width:usize, height:usize, fill_value:f32, mode:enums::ImageMode) -> error::Result<ImageBuffer> {

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, fill_value);

        Ok(ImageBuffer{buffer:v,
            width,
            height,
            empty:false,
            mask:None,
            mode: mode
        })
    }

    // Creates a new image buffer of the requested width and height
    pub fn new_with_mask(width:usize, height:usize, mask:&Option<Vec<bool>>) -> error::Result<ImageBuffer> {

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        Ok(ImageBuffer{buffer:v,
            width,
            height,
            empty:false,
            mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
            mode: enums::ImageMode::U16BIT
        })
    }

    fn new_with_mask_as_mode(width:usize, height:usize, mask:&Option<Vec<bool>>, mode:enums::ImageMode) -> error::Result<ImageBuffer> {

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        Ok(ImageBuffer{buffer:v,
            width,
            height,
            empty:false,
            mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
            mode: mode
        })
    }


    pub fn new_empty() -> error::Result<ImageBuffer> {
        Ok(ImageBuffer{buffer:Vec::new(),
            width:0,
            height:0,
            empty:true,
            mask:None,
            mode: enums::ImageMode::U16BIT
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec(v:Vec<f32>, width:usize, height:usize) -> error::Result<ImageBuffer> {
        ImageBuffer::from_vec_as_mode(v, width, height, enums::ImageMode::U16BIT)
    }


        // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_as_mode(v:Vec<f32>, width:usize, height:usize, mode:enums::ImageMode) -> error::Result<ImageBuffer> {

        if v.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        Ok(ImageBuffer{buffer:v,
                    width,
                    height,
                    empty:false,
                    mask:None,
                    mode: mode
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u8(v_u8:&Vec<u8>, width:usize, height:usize) -> error::Result<ImageBuffer> {

        if v_u8.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        let mut v = vec![0.0_f32; width * height];
        for i in 0..v_u8.len() {
            v[i] = v_u8[i] as f32;
        }

        Ok(ImageBuffer{buffer:v,
                    width,
                    height,
                    empty:false,
                    mask:None,
                    mode: enums::ImageMode::U16BIT
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u8_with_mask(v_u8:&Vec<u8>, width:usize, height:usize, mask:&Option<Vec<bool>>) -> error::Result<ImageBuffer> {

        if v_u8.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        let mut v = vec![0.0_f32; width * height];
        for i in 0..v_u8.len() {
            v[i] = v_u8[i] as f32;
        }

        Ok(ImageBuffer{buffer:v,
                    width:width,
                    height:height,
                    empty:false,
                    mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
                    mode: enums::ImageMode::U16BIT
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u16(v_u16:&Vec<u16>, width:usize, height:usize) -> error::Result<ImageBuffer> {

        if v_u16.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        let mut v = vec![0.0_f32; width * height];
        for i in 0..v_u16.len() {
            v[i] = v_u16[i] as f32;
        }

        Ok(ImageBuffer{buffer:v,
                    width,
                    height,
                    empty:false,
                    mask:None,
                    mode: enums::ImageMode::U16BIT
        })
    }

        // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_u16_with_mask(v_u16:&Vec<u16>, width:usize, height:usize, mask:&Option<Vec<bool>>) -> error::Result<ImageBuffer> {

        if v_u16.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        let mut v = vec![0.0_f32; width * height];
        for i in 0..v_u16.len() {
            v[i] = v_u16[i] as f32;
        }

        Ok(ImageBuffer{buffer:v,
                    width:width,
                    height:height,
                    empty:false,
                    mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
                    mode: enums::ImageMode::U16BIT
        })
    }

    // Creates a new image buffer at the requested width, height and data
    pub fn from_vec_with_mask(v:Vec<f32>, width:usize, height:usize, mask:&Option<Vec<bool>>) -> error::Result<ImageBuffer> {

        if v.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        Ok(ImageBuffer{buffer:v,
                    width,
                    height,
                    empty:false,
                    mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
                    mode: enums::ImageMode::U16BIT
        })
    }


    pub fn from_image_u8(image_data:&image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>>) -> error::Result<ImageBuffer> {
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let value = pixel[0] as f32;
                let idx = y * width + x;
                v[idx] = value;
            }
        }

        ImageBuffer::from_vec_as_mode(v, width, height, enums::ImageMode::U16BIT)
    }

    pub fn from_image_u16(image_data:&image::ImageBuffer<image::Rgba<u16>, std::vec::Vec<u16>>) -> error::Result<ImageBuffer> {
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let value = pixel[0] as f32;
                let idx = y * width + x;
                v[idx] = value;
            }
        }

        ImageBuffer::from_vec_as_mode(v, width, height, enums::ImageMode::U16BIT)
    }


    pub fn from_file(file_path:&str) -> error::Result<ImageBuffer> {

        if !path::file_exists(file_path) {
            panic!("File not found: {}", file_path);
        }

        let image_data = open(file_path).unwrap().into_luma16();
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        let mut v:Vec<f32> = Vec::with_capacity(width * height);
        v.resize(width * height, 0.0);

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let value = pixel[0] as f32;
                let idx = y * width + x;
                v[idx] = value;
            }
        }

        ImageBuffer::from_vec(v, width, height)
    }

    fn new_from_op(v:Vec<f32>, width:usize, height:usize, mask:&Option<Vec<bool>>, mode:enums::ImageMode) -> error::Result<ImageBuffer> {
        if v.len() != (width * height) {
            panic!("Dimensions to not match vector length");
        }

        Ok(ImageBuffer{buffer:v,
            width,
            height,
            empty:false,
            mask: if *mask != None { Some(mask.as_ref().unwrap().to_owned()) } else { None },
            mode: mode
        })
    }

    fn buffer_to_mask(&self, buffer:&ImageBuffer) -> error::Result<Vec<bool>> {
        if buffer.width != self.width || buffer.height != self.height {
            panic!("Array size mismatch");
        }

        let mut m : Vec<bool> = Vec::with_capacity(self.buffer.len());
        m.resize(self.buffer.len(), false);

        for i in 0..self.buffer.len() {
            m[i] = buffer.buffer[i] > 0.0;
        }

        Ok(m)
    }

    pub fn set_mask(&mut self, mask:&ImageBuffer) {
        self.mask = Some(self.buffer_to_mask(&mask).unwrap());
    }

    pub fn copy_mask_to(&self, dest:&mut ImageBuffer) {
        dest.mask = self.mask.to_owned();
    }

    pub fn clear_mask(&mut self) {
        self.mask = None;
    }

    fn get_mask_at_index(&self, idx:usize) -> error::Result<bool> {
        match &self.mask {
            Some(b) => {
                if idx >= b.len() {
                    panic!("Invalid pixel coordinates");
                }
                Ok(b[idx])
            },
            None => Ok(true)
        }
    }

    pub fn get_mask_at_point(&self, x:usize, y:usize) -> error::Result<bool> {
        match &self.mask {
            Some(b) => {
                if x >= self.width || y >= self.height {
                    panic!("Invalid pixel coordinates");
                }
                let msk_idx = self.width * y + x;
                Ok(b[msk_idx])
            },
            None => Ok(true)
        }
    }

    pub fn get_slice(&self, top_y:usize, len:usize) -> error::Result<ImageBuffer> {
        let start_index = top_y * self.width;
        let stop_index = (top_y + len) * self.width;

        let slice = self.buffer[start_index..stop_index].to_vec();

        ImageBuffer::from_vec(slice, self.width, len)
    }

    pub fn to_vector_u8(&self) -> Vec<u8> {
        let need_len = self.buffer.len();
        let mut v:Vec<u8> = Vec::with_capacity(need_len);
        v.resize(need_len, 0);

        for i in 0..need_len {
            v[i] = match self.mode {
                enums::ImageMode::U8BIT => self.buffer[i] as u8,
                enums::ImageMode::U12BIT => (self.buffer[i] / 2033.0 * 255.0) as u8,
                enums::ImageMode::U16BIT => (self.buffer[i] / 65535.0 * 255.0) as u8
                
            }
        }
        v
    }

    pub fn to_vector_u16(&self) -> Vec<u16> {
        let need_len = self.buffer.len();
        let mut v:Vec<u16> = Vec::with_capacity(need_len);
        v.resize(need_len, 0);

        for i in 0..need_len {
            v[i] = self.buffer[i] as u16;
        }
        v
    }

    pub fn to_vector(&self) -> Vec<f32> {
        self.buffer.clone()
    }

    pub fn get_subframe(&self, left_x:usize, top_y:usize, width:usize, height:usize) -> error::Result<ImageBuffer> {

        let subframed_buffer = subframe_array(&self.buffer, self.width, self.height, left_x, top_y, width, height);
        let subframed_mask = match &self.mask {
            Some(m) => Some(subframe_array(&m, self.width, self.height, left_x, top_y, width, height)),
            None => None,
        };
        ImageBuffer::new_from_op(subframed_buffer, width, height, &subframed_mask, self.mode)
    }

    pub fn isolate_window(&self, window_size:usize, x:usize, y:usize) -> Vec<f32> {
        let mut v:Vec<f32> = Vec::with_capacity(window_size * window_size);
        let start = window_size as i32 / 2 * -1;
        let end = window_size as i32 / 2 + 1;
        for _y in start..end as i32 {
            for _x in start..end as i32 {
                let get_x = x as i32 + _x;
                let get_y = y as i32 + _y;
                if get_x >= 0 
                    && get_x < self.width as i32 
                    && get_y >= 0 
                    && get_y < self.height as i32
                    && self.get_mask_at_point(get_x as usize, get_y as usize).unwrap()
                    {
                    v.push(self.get(get_x as usize, get_y as usize).unwrap());
                }
            }
        }
        v
    }

    pub fn get(&self, x:usize, y:usize) -> error::Result<f32> {
        if x < self.width && y < self.height {
            if ! self.get_mask_at_point(x, y).unwrap() {
                return Ok(0.0);
            }
            let index = y * self.width + x;
            Ok(self.buffer[index])
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    pub fn get_interpolated(&self, x:f32, y:f32)-> error::Result<f32> {
        if x < self.width as f32 && y < self.height as f32 {

            let xf = x.floor();
            let xc = xf + 1.0;

            let yf = y.floor();
            let yc = yf + 1.0;

            let xd = x - xf;
            let yd = y - yf;

            let v00 = self.get(xf as usize, yf as usize).unwrap();
            let v01 = self.get(xc as usize, yf as usize).unwrap();
            let v10 = self.get(xf as usize, yc as usize).unwrap();
            let v11 = self.get(xc as usize, yc as usize).unwrap();

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

    pub fn put_u16(&mut self, x:usize, y:usize, val:u16) {
        self.put(x, y, val as f32);
    }

    pub fn put(&mut self, x:usize, y:usize, val:f32) {
        if x < self.width && y < self.height {
            if self.get_mask_at_point(x, y).unwrap() {
                let index = y * self.width + x;
                self.buffer[index] = val;
            }
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    // Computes the mean of all pixel values
    pub fn mean(&self) -> f32 {

        let mut total:f32 = 0.0;
        let mut count:f32 = 0.0;

        // It is *soooo* inefficient to keep doing this...
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let pixel_value = self.get(x, y).unwrap();
                    if pixel_value > 0.0 {
                        total += pixel_value;
                        count += 1.0;
                    }
                }   
            }
        }

        if count > 0.0 { // Prevent divide-by-zero on fully-masked images
            total / count
        } else {
            0.0
        }
    }

    pub fn divide_mut(&mut self, other:&ImageBuffer) {

        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = if other.buffer[i] != 0.0 { self.buffer[i] / other.buffer[i] } else { 0.0 };
            }
        }
    }

    pub fn divide(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let quotient = if other.buffer[i] != 0.0 { self.buffer[i] / other.buffer[i] } else { 0.0 };
                v[i] = quotient;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }

    pub fn divide_into_mut(&mut self, divisor:f32) {
        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = if self.buffer[i] != 0.0 { divisor / self.buffer[i] } else { 0.0 };
            }
        }
    }

    pub fn divide_into(&self, divisor:f32) -> error::Result<ImageBuffer> {
        
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let quotient = if self.buffer[i] != 0.0 { divisor / self.buffer[i] } else { 0.0 };
                v[i] = quotient;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }

    pub fn scale_mut(&mut self, scalar:f32) {
        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = self.buffer[i] * scalar;
            }
        }
    }

    pub fn scale(&self, scalar:f32) -> error::Result<ImageBuffer> {
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let product = self.buffer[i] * scalar;
                v[i] = product;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }

    pub fn multiply_mut(&mut self, other:&ImageBuffer) {

        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = self.buffer[i] * other.buffer[i];
            }
        }
    }

    pub fn multiply(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let product = self.buffer[i] * other.buffer[i];
                v[i] = product;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }

    pub fn add_mut(&mut self, other:&ImageBuffer) {
        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = self.buffer[i] + other.buffer[i];
            }
        }
    }

    pub fn add_across_mut(&mut self, other:f32) {
        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = self.buffer[i] + other;
            }
        }
    }

    pub fn add(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let result = self.buffer[i] + other.buffer[i];
                v[i] = result;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }

    pub fn subtract_mut(&mut self, other:&ImageBuffer) {
        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = self.buffer[i] - other.buffer[i];
            }
        }
    }

    pub fn subtract_across_mut(&mut self, other:f32) {
        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = self.buffer[i] - other;
            }
        }
    }

    pub fn subtract(&self, other:&ImageBuffer) -> error::Result<ImageBuffer> {

        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let difference = self.buffer[i] - other.buffer[i];
                v[i] = difference;
            }
        }

        ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode)
    }


    pub fn shift_to_min_zero(&self) -> error::Result<ImageBuffer> {

        let minmax = self.get_min_max().unwrap();

        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let value = self.buffer[i];
                if minmax.min < 0.0 {
                    v[i] = value + minmax.min;
                } else {
                    v[i] = value - minmax.min;
                }
            }
        }

        Ok(ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode).unwrap())
    }

    pub fn normalize_force_minmax(&self, min:f32, max:f32, forced_min:f32, forced_max:f32) -> error::Result<ImageBuffer> {
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let value = ((self.buffer[i] - forced_min) / (forced_max- forced_min)) * (max - min) + min;
                v[i] = value;
            }
        }

        Ok(ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode).unwrap())
    }

    pub fn normalize(&self, min:f32, max:f32) -> error::Result<ImageBuffer> {
        let minmax = self.get_min_max().unwrap();
        self.normalize_force_minmax(min, max, minmax.min, minmax.max)
    }

    pub fn normalize_mut(&mut self, min:f32, max:f32) {
        self.buffer = self.normalize(min, max).unwrap().buffer;
    }


    pub fn crop(&self, height:usize, width:usize) -> error::Result<ImageBuffer> {
        let cropped_buffer = crop_array(&self.buffer, self.width, self.height, width, height);

        let cropped_mask = match &self.mask {
            Some(m) => Some(crop_array(&m, self.width, self.height, width, height)),
            None => None,
        };
        ImageBuffer::new_from_op(cropped_buffer, width, height, &cropped_mask, self.mode)
    }

    pub fn clip(&self, clip_min:f32, clip_max:f32) -> error::Result<ImageBuffer> {
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let value = self.buffer[i];
                if value < clip_min {
                    v[i] = clip_min;
                } else if value > clip_max {
                    v[i] = clip_max;
                } else {
                    v[i] = value;
                }
            }
        }

        Ok(ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode).unwrap())
    }

    pub fn clip_mut(&mut self, clip_min:f32, clip_max:f32) {
        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                let value = self.buffer[i];
                if value < clip_min {
                    self.buffer[i] = clip_min;
                } else if value > clip_max {
                    self.buffer[i] = clip_max;
                } 
            }
        }
    }

    pub fn power(&self, power:f32) -> error::Result<ImageBuffer> {
        let need_len = self.width * self.height;
        let mut v:Vec<f32> = Vec::with_capacity(need_len);
        v.resize(need_len, 0.0);

        for i in 0..need_len {
            if self.get_mask_at_index(i).unwrap() {
                let value = self.buffer[i];
                v[i] = value.powf(power);
            }
        }

        Ok(ImageBuffer::new_from_op(v, self.width, self.height, &self.mask, self.mode).unwrap())
    }

    pub fn power_mut(&mut self, power:f32) {
        for i in 0..self.buffer.len() {
            if self.get_mask_at_index(i).unwrap() {
                self.buffer[i] = self.buffer[i].powf(power);
            }
        }
    }

    pub fn calc_center_of_mass_offset(&self, threshold:f32) -> Offset {
        let mut ox: f32 = 0.0;
        let mut oy: f32 = 0.0;
        let mut count: u32 = 0;
    
        for y in 0..self.height {
            for x in 0..self.width {
                let val = self.get(x, y).unwrap();
                if val >= threshold {
                    ox = ox + (x as f32);
                    oy = oy + (y as f32);
                    count = count + 1;
                }   
            }
        }
    
        if count > 0 {
            ox = (self.width as f32 / 2.0) - (ox / (count as f32));
            oy = (self.height as f32 / 2.0) - (oy / (count as f32));
        }
    
        Offset{
            h:ox, 
            v:oy
        }
    }

    pub fn paste_mut(&mut self, src:&ImageBuffer, tl_x:usize, tl_y:usize) {
        for y in 0..src.height {
            for x in 0..src.width {

                let dest_x = tl_x + x;
                let dest_y = tl_y + y;

                self.put(dest_x, dest_y, src.get(x, y).unwrap());
            }
        }
    }

    pub fn paste(&self, src:&ImageBuffer, tl_x:usize, tl_y:usize) -> error::Result<ImageBuffer> {
        let mut new_buffer = ImageBuffer::from_vec_with_mask(self.buffer.clone(), self.width, self.height, &self.mask).unwrap();
        for y in 0..src.height {
            for x in 0..src.width {

                let dest_x = tl_x + x;
                let dest_y = tl_y + y;

                new_buffer.put(dest_x, dest_y, src.get(x, y).unwrap());
            }
        }
        Ok(new_buffer)
    }


    pub fn shift(&self, horiz:i32, vert:i32) -> error::Result<ImageBuffer> {

        let mut shifted_buffer = ImageBuffer::new_with_mask(self.width, self.height, &self.mask).unwrap();

        let h = self.height as i32;
        let w = self.width as i32;

        for y in 0..h {
            for x in 0..w {
                if self.get_mask_at_point(x as usize, y as usize).unwrap() {
                    let shift_x = x as i32 + horiz;
                    let shift_y = y as i32 + vert;
                
                    if shift_x >= 0 && shift_y >= 0 && shift_x < w  && shift_y < h {
                        shifted_buffer.put(shift_x as usize, shift_y as usize, self.get(x as usize, y as usize).unwrap());
                    }
                }
            }
        }
        Ok(shifted_buffer)
    }

    pub fn shift_interpolated(&self, horiz:f32, vert:f32) -> error::Result<ImageBuffer> {
        let mut shifted_buffer = ImageBuffer::new_with_mask(self.width, self.height, &self.mask).unwrap();

        let h = self.height as i32 - 1;
        let w = self.width as i32 - 1;

        let hf = horiz - horiz.floor();
        let vf = vert - vert.floor();

        for y in 0..h {
            for x in 0..w {
                if self.get_mask_at_point(x as usize, y as usize).unwrap() {
                    let shift_x = x as f32 + horiz.floor();
                    let shift_y = y as f32 + vert.floor();
                
                    if shift_x >= 0.0 && shift_y >= 0.0 && shift_x < w as f32  && shift_y < h as f32 {
                        shifted_buffer.put(shift_x as usize, shift_y as usize, self.get_interpolated(x as f32 - (1.0 - hf), y as f32 - (1.0 - vf)).unwrap());
                    }
                }
            }
        }
        Ok(shifted_buffer)
    }

    pub fn get_min_max(&self) -> error::Result<MinMax> {
        
        let mut mx:f32 = std::f32::MIN;
        let mut mn:f32 = std::f32::MAX;

        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let val = self.get(x, y).unwrap() as f32;
                    mx = if val > mx { val } else { mx };
                    mn = if val < mn { val } else { mn };
                }
            }
        }
        
        Ok(MinMax{min:mn, max:mx})
    }


    pub fn buffer_to_image_8bit(&self) -> image::ImageBuffer<image::Rgba<u8>, std::vec::Vec<u8>> {
        let mut out_img = DynamicImage::new_rgba8(self.width as u32, self.height as u32).into_rgba8();
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let val = self.get(x, y).unwrap().round() as u8;
                    let a = if self.get_mask_at_point(x, y).unwrap() { 255 } else { 0 };
                    out_img.put_pixel(x as u32, y as u32, Rgba([val, val, val, a]));
                }
            }
        }

        out_img
    }

    pub fn buffer_to_image_16bit(&self) -> image::ImageBuffer<image::Rgba<u16>, std::vec::Vec<u16>>
    {
        let mut out_img = DynamicImage::new_rgba16(self.width as u32, self.height as u32).into_rgba16();
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let val = self.get(x, y).unwrap().round() as u16;
                    let a = if self.get_mask_at_point(x, y).unwrap() { 65535 } else { 0 };
                    out_img.put_pixel(x as u32, y as u32, Rgba([val, val, val, a]));
                }
            }
        }

        out_img
    }


    pub fn save_16bit(&self, to_file:&str) {
        let mut out_img = DynamicImage::new_rgba16(self.width as u32, self.height as u32).into_rgba16();
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let val = self.get(x, y).unwrap().round() as u16;
                    let a = if self.get_mask_at_point(x, y).unwrap() { 65535 } else { 0 };
                    out_img.put_pixel(x as u32, y as u32, Rgba([val, val, val, a]));
                }
            }
        }

        if path::parent_exists_and_writable(&to_file) {
            out_img.save(to_file).unwrap();
        } else {
            panic!("Parent path does not exist or is unwritable: {}", path::get_parent(to_file));
        }
    
    }

    pub fn save_8bit(&self, to_file:&str) {
        let mut out_img = DynamicImage::new_rgba8(self.width as u32, self.height as u32).into_rgba8();
        
        for y in 0..self.height {
            for x in 0..self.width {
                if self.get_mask_at_point(x, y).unwrap() {
                    let val = self.get(x, y).unwrap().round() as u8;
                    let a = if self.get_mask_at_point(x, y).unwrap() { 255 } else { 0 };
                    out_img.put_pixel(x as u32, y as u32, Rgba([val, val, val, a]));
                }
            }
        }

        if path::parent_exists_and_writable(&to_file) {
            out_img.save(to_file).unwrap();
        } else {
            panic!("Parent path does not exist or is unwritable: {}", path::get_parent(to_file));
        }
    
    }

    pub fn save(&self, to_file:&str, mode:enums::ImageMode) {
        match mode {
            enums::ImageMode::U8BIT => {
                self.save_8bit(to_file)
            },
            _ => {
                self.save_16bit(to_file)
            }
        };
    }
}

