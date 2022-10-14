use crate::{
    debayer, decompanding, enums, error, hotpixel, imagebuffer::ImageBuffer, imagebuffer::Offset,
    imagerot, inpaint, max, min, noise, path, Mask, MaskVec,
};

use image::{open, DynamicImage, Rgba};

// A simple image raster buffer.
#[derive(Debug, Clone)]
pub struct RgbImage {
    bands: Vec<ImageBuffer>,
    alpha: MaskVec, // Intended to work as an alpha transparency band
    uses_alpha: bool,
    pub width: usize,
    pub height: usize,
    mode: enums::ImageMode,
    empty: bool,
}

macro_rules! check_band_in_bounds {
    ($band:expr, $self:ident) => {
        if $band >= $self.bands.len() {
            panic!("Band index out of bounds: {}", $band);
        }
    };
}

#[allow(dead_code)]
impl RgbImage {
    pub fn new(width: usize, height: usize, mode: enums::ImageMode) -> error::Result<RgbImage> {
        Ok(RgbImage {
            bands: vec![],
            alpha: MaskVec::new(),
            uses_alpha: false,
            width,
            height,
            mode,
            empty: false,
        })
    }

    pub fn new_with_bands(
        width: usize,
        height: usize,
        num_bands: usize,
        mode: enums::ImageMode,
    ) -> error::Result<RgbImage> {
        let mut bands: Vec<ImageBuffer> = vec![];
        for _ in 0..num_bands {
            bands.push(ImageBuffer::new(width, height).unwrap());
        }
        Ok(RgbImage {
            bands,
            alpha: MaskVec::new(),
            uses_alpha: false,
            width,
            height,
            mode,
            empty: false,
        })
    }

    pub fn new_with_bands_masked(
        width: usize,
        height: usize,
        num_bands: usize,
        mode: enums::ImageMode,
        mask_value: bool,
    ) -> error::Result<RgbImage> {
        let mut bands: Vec<ImageBuffer> = vec![];
        for _ in 0..num_bands {
            bands.push(ImageBuffer::new(width, height).unwrap());
        }

        Ok(RgbImage {
            bands,
            alpha: MaskVec::fill_mask(width * height, mask_value),
            uses_alpha: true,
            width,
            height,
            mode,
            empty: false,
        })
    }

    pub fn new_empty() -> error::Result<RgbImage> {
        Ok(RgbImage {
            bands: vec![],
            alpha: MaskVec::new(),
            uses_alpha: false,
            width: 0,
            height: 0,
            mode: enums::ImageMode::U8BIT,
            empty: true,
        })
    }

    pub fn open_str(file_path: &str) -> error::Result<RgbImage> {
        RgbImage::open(&String::from(file_path))
    }

    pub fn open16(file_path: &String) -> error::Result<RgbImage> {
        if !path::file_exists(file_path.as_str()) {
            panic!("File not found: {}", file_path);
        }

        let image_data = open(&file_path).unwrap().into_rgba16();
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        let mut rgbimage =
            RgbImage::new_with_bands_masked(width, height, 3, enums::ImageMode::U16BIT, true)
                .unwrap();

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                //println!("Pixel: {}", pixel.len());
                let red = pixel[0] as f32;
                let green = pixel[1] as f32;
                let blue = pixel[2] as f32;
                let alpha: f32 = pixel[3] as f32;

                rgbimage.put(x, y, red, 0);
                rgbimage.put(x, y, green, 1);
                rgbimage.put(x, y, blue, 2);

                rgbimage.put_alpha(x, y, alpha > 0.0);
            }
        }

        Ok(rgbimage)
    }

    pub fn open(file_path: &String) -> error::Result<RgbImage> {
        if !path::file_exists(file_path.as_str()) {
            panic!("File not found: {}", file_path);
        }

        let image_data = open(&file_path).unwrap().into_rgba8();
        let dims = image_data.dimensions();

        let width = dims.0 as usize;
        let height = dims.1 as usize;

        // TODO: Don't assume 8-bit
        let mut rgbimage =
            RgbImage::new_with_bands_masked(width, height, 3, enums::ImageMode::U8BIT, true)
                .unwrap();

        for y in 0..height {
            for x in 0..width {
                let pixel = image_data.get_pixel(x as u32, y as u32);
                let red = pixel[0] as f32;
                let green = pixel[1] as f32;
                let blue = pixel[2] as f32;
                let alpha: f32 = pixel[3] as f32;

                rgbimage.put(x, y, red, 0);
                rgbimage.put(x, y, green, 1);
                rgbimage.put(x, y, blue, 2);

                rgbimage.put_alpha(x, y, alpha > 0.0);
            }
        }

        Ok(rgbimage)
    }

    pub fn new_from_buffers_rgb(
        red: &ImageBuffer,
        green: &ImageBuffer,
        blue: &ImageBuffer,
        mode: enums::ImageMode,
    ) -> error::Result<RgbImage> {
        Ok(RgbImage {
            bands: vec![red.clone(), green.clone(), blue.clone()],
            alpha: MaskVec::new(),
            uses_alpha: false,
            width: red.width,
            height: red.height,
            mode,
            empty: false,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.empty
    }

    pub fn get_mode(&self) -> enums::ImageMode {
        self.mode
    }

    pub fn set_mode(&mut self, mode: enums::ImageMode) {
        self.mode = mode;
    }

    pub fn num_bands(&self) -> usize {
        self.bands.len()
    }

    pub fn get_band(&self, band: usize) -> &ImageBuffer {
        check_band_in_bounds!(band, self);
        &self.bands[band]
    }

    pub fn set_band(&mut self, buffer: &ImageBuffer, band: usize) {
        if self.bands.len() <= band {
            panic!("Invalid band specified");
        }

        self.bands[band] = buffer.clone();
    }

    pub fn push_band(&mut self, buffer: &ImageBuffer) -> usize {
        self.bands.push(buffer.clone());
        self.num_bands()
    }

    pub fn divide_from_each(&mut self, other: &ImageBuffer) {
        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        for i in 0..self.bands.len() {
            self.bands[i].divide_mut(other);
        }
    }

    pub fn add_to_each(&mut self, other: &ImageBuffer) {
        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        for i in 0..self.bands.len() {
            self.bands[i].add_mut(other);
        }
    }

    pub fn add(&mut self, other: &RgbImage) {
        if self.width != other.width || self.height != other.height {
            panic!("Array size mismatch");
        }

        for i in 0..self.bands.len() {
            // Handle adding a mono 'other' to a multi-band self.
            let other_band = match other.num_bands() > i {
                true => i,
                false => 0,
            };

            self.bands[i].add_mut(other.get_band(other_band));
        }
    }

    pub fn levels(&mut self, black_level: f32, white_level: f32, gamma: f32) {
        for b in 0..self.bands.len() {
            let mm = self.bands[b].get_min_max();

            let rng = match self.mode {
                enums::ImageMode::U8BIT => 256.0,
                enums::ImageMode::U16BIT => 65535.0,
                enums::ImageMode::U12BIT => 2033.0, // I know, not really. Will need to adjust later for NSYT ILT
            };

            let norm_min = (rng * black_level) + mm.min;
            let norm_max = (rng * white_level) + mm.min;

            self.bands[b].clip_mut(norm_min, norm_max);
            self.bands[b].power_mut(gamma);
            self.bands[b] = self.bands[b].normalize(mm.min, mm.max).unwrap();
        }
    }

    pub fn put(&mut self, x: usize, y: usize, value: f32, band: usize) {
        if x < self.width && y < self.height {
            self.bands[band].put(x, y, value);
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    pub fn put_alpha(&mut self, x: usize, y: usize, value: bool) {
        if x < self.width && y < self.height {
            self.alpha.put_2d(self.width, self.height, x, y, value);
        } else {
            panic!("Invalid pixel coordinates");
        }
    }

    pub fn paste(&mut self, src: &RgbImage, tl_x: usize, tl_y: usize) {
        for i in 0..self.bands.len() {
            self.bands[i].paste_mut(src.get_band(i), tl_x, tl_y);
        }
    }

    pub fn get_alpha_at(&self, x: usize, y: usize) -> bool {
        if self.uses_alpha {
            self.alpha.get_2d(self.width, self.height, x, y)
        } else {
            true
        }
    }

    // Doesn't do anything if alpha is already enabled.
    pub fn init_alpha(&mut self) {
        if !self.uses_alpha {
            self.uses_alpha = true;
            self.alpha = MaskVec::new_mask(self.width * self.height);
        }
    }

    pub fn clear_alpha(&mut self) {
        if self.uses_alpha {
            self.alpha.clear_mask();
        }
    }

    pub fn is_using_alpha(&self) -> bool {
        self.uses_alpha
    }

    pub fn apply_mask_to_band(&mut self, mask: &ImageBuffer, band: usize) {
        check_band_in_bounds!(band, self);
        self.bands[band].set_mask(mask);
    }

    pub fn clear_mask_on_band(&mut self, band: usize) {
        check_band_in_bounds!(band, self);
        self.bands[band].clear_mask();
    }

    pub fn copy_alpha_from(&mut self, src: &ImageBuffer) {
        self.alpha = ImageBuffer::buffer_to_mask(src);
    }

    pub fn calibrate_band(
        &mut self,
        band: usize,
        flat_field: &RgbImage,
        dark_field: &RgbImage,
        dark_flat_field: &RgbImage,
    ) {
        check_band_in_bounds!(band, self);

        if !flat_field.is_empty() && !dark_field.is_empty() && !dark_flat_field.is_empty() {
            let flat_minus_darkflat = flat_field.bands[band]
                .subtract(&dark_flat_field.bands[band])
                .unwrap();
            let darkflat = flat_minus_darkflat
                .subtract(&dark_field.bands[band])
                .unwrap();
            let mean_flat = darkflat.mean();
            let frame_minus_dark = self.bands[band].subtract(&dark_field.bands[band]).unwrap();
            self.bands[band] = frame_minus_dark
                .scale(mean_flat)
                .unwrap()
                .divide(&flat_minus_darkflat)
                .unwrap();
        } else if !flat_field.is_empty() && !dark_field.is_empty() {
            let darkflat = flat_field.bands[band]
                .subtract(&dark_field.bands[band])
                .unwrap();
            let mean_flat = darkflat.mean();
            let frame_minus_dark = self.bands[band].subtract(&dark_field.bands[band]).unwrap();
            self.bands[band] = frame_minus_dark
                .scale(mean_flat)
                .unwrap()
                .divide(&flat_field.bands[band])
                .unwrap();
        } else if !flat_field.is_empty() && dark_field.is_empty() {
            let mean_flat = flat_field.bands[band].mean();
            self.bands[band] = self.bands[band]
                .scale(mean_flat)
                .unwrap()
                .divide(&flat_field.bands[band])
                .unwrap();
        } else if flat_field.is_empty() && !dark_field.is_empty() {
            self.bands[band] = self.bands[band].subtract(&dark_field.bands[band]).unwrap();
        }
    }

    pub fn calibrate(
        &mut self,
        flat_field: &RgbImage,
        dark_field: &RgbImage,
        dark_flat_field: &RgbImage,
    ) {
        for i in 0..self.bands.len() {
            self.calibrate_band(i, flat_field, dark_field, dark_flat_field);
        }
    }

    fn apply_flat_on_band(&mut self, band: usize, flat_buffer: &ImageBuffer) {
        let mean_flat = flat_buffer.mean();
        self.bands[band] = self.bands[band]
            .scale(mean_flat)
            .unwrap()
            .divide(flat_buffer)
            .unwrap();
    }

    pub fn apply_flat(&mut self, flat: &RgbImage) {
        for i in 0..self.bands.len() {
            let flat_buffer = if flat.num_bands() > i {
                flat.get_band(i)
            } else {
                flat.get_band(0)
            };

            self.apply_flat_on_band(i, flat_buffer);
        }
    }

    pub fn flatfield(&mut self, flat: &RgbImage) {
        self.apply_flat(flat);
    }

    pub fn calc_center_of_mass_offset(&self, threshold: f32, band: usize) -> Offset {
        check_band_in_bounds!(band, self);

        self.bands[band].calc_center_of_mass_offset(threshold)
    }

    pub fn shift_band(&mut self, horiz: f32, vert: f32, band: usize) {
        check_band_in_bounds!(band, self);

        // Shifting using fractional amounts. It's not perfectly implemented yet, but I'll leave it until I think of how to improve it.
        self.bands[band] = self.bands[band].shift_interpolated(horiz, vert).unwrap();
    }

    pub fn shift(&mut self, horiz: f32, vert: f32) {
        for i in 0..self.bands.len() {
            self.shift_band(horiz, vert, i);
        }
    }

    pub fn compand(&mut self, ilt: &[u32; 256]) {
        for i in 0..self.bands.len() {
            decompanding::compand_buffer(&mut self.bands[i], ilt);
        }
        self.mode = enums::ImageMode::U8BIT;
    }

    pub fn decompand(&mut self, ilt: &[u32; 256]) {
        for i in 0..self.bands.len() {
            decompanding::decompand_buffer(&mut self.bands[i], ilt);
        }
        self.mode = enums::ImageMode::U12BIT;
    }

    pub fn debayer(&mut self) {
        let use_band = 0;
        check_band_in_bounds!(use_band, self);

        let debayered = debayer::debayer(&self.bands[use_band]).unwrap();
        self.bands = vec![
            debayered.bands[0].clone(),
            debayered.bands[1].clone(),
            debayered.bands[2].clone(),
        ];
    }

    pub fn reduce_color_noise(&mut self, amount: i32) {
        let orig_mode = self.mode;
        let (_, maxval) = self.get_min_max_all_channel();
        self.normalize_to_8bit_with_max(maxval);

        let result = noise::color_noise_reduction(&mut self.clone(), amount);
        for i in 0..self.bands.len() {
            self.bands[i] = result.bands[i].clone();
        }

        if orig_mode == enums::ImageMode::U12BIT {
            self.normalize_to_12bit_with_max(maxval, 255.0);
        } else if orig_mode == enums::ImageMode::U16BIT {
            self.normalize_to_16bit_with_max(255.0);
        }
    }

    pub fn apply_weight_on_band(&mut self, scalar: f32, band: usize) {
        check_band_in_bounds!(band, self);
        self.bands[band].scale_mut(scalar);
    }

    pub fn hot_pixel_correction_on_band(&mut self, window_size: i32, threshold: f32, band: usize) {
        check_band_in_bounds!(band, self);
        self.bands[band] = hotpixel::hot_pixel_detection(&self.bands[band], window_size, threshold)
            .unwrap()
            .buffer;
    }

    pub fn hot_pixel_correction(&mut self, window_size: i32, threshold: f32) {
        for i in 0..self.bands.len() {
            self.hot_pixel_correction_on_band(window_size, threshold, i);
        }
    }

    pub fn crop(&mut self, x: usize, y: usize, width: usize, height: usize) {
        for i in 0..self.bands.len() {
            self.bands[i] = self.bands[i].get_subframe(x, y, width, height).unwrap();
        }
        self.width = width;
        self.height = height;
    }

    pub fn rotate_band(&mut self, rotation_radians: f32, band: usize) {
        if band >= self.bands.len() {
            panic!("Band index {} out of bounds", band);
        }

        self.bands[band] =
            imagerot::rotate(&self.bands[band], rotation_radians).expect("Error rotating image");
    }

    pub fn rotate(&mut self, rotation_radians: f32) {
        for i in 0..self.bands.len() {
            self.rotate_band(rotation_radians, i);
        }
    }

    fn is_pixel_grayscale(&self, x: usize, y: usize) -> bool {
        if self.bands.len() <= 1 {
            return true;
        }

        let mut v = std::f32::MIN;

        for i in 0..self.bands.len() {
            let b = self.bands[i].get(x, y).unwrap();
            if v == std::f32::MIN {
                v = b;
            } else if v != b {
                return false;
            }
        }

        true
    }

    // This makes some assumptions and isn't perfect.
    pub fn is_grayscale(&self) -> bool {
        let tl = self.is_pixel_grayscale(30, 30);
        let bl = self.is_pixel_grayscale(30, self.height - 30);
        let tr = self.is_pixel_grayscale(self.width - 30, 30);
        let br = self.is_pixel_grayscale(self.width - 30, self.height - 30);

        let mid_x = self.width / 2;
        let mid_y = self.height / 2;

        let mtl = self.is_pixel_grayscale(mid_x - 20, mid_y - 20);
        let mbl = self.is_pixel_grayscale(mid_x - 20, mid_y + 20);
        let mtr = self.is_pixel_grayscale(mid_x + 20, mid_y - 20);
        let mbr = self.is_pixel_grayscale(mid_x + 20, mid_y + 20);

        tl && bl && tr && br && mtl && mbl && mtr && mbr
    }

    pub fn apply_inpaint_fix(&mut self, mask: &ImageBuffer) {
        let fixed = inpaint::apply_inpaint_to_buffer(self, mask).unwrap();
        self.bands = fixed.bands;

        // let mut new_r = fixed.red().clone();
        // self._red.copy_mask_to(&mut new_r);

        // let mut new_g = fixed.green().clone();
        // self._green.copy_mask_to(&mut new_g);

        // let mut new_b = fixed.blue().clone();
        // self._blue.copy_mask_to(&mut new_b);

        // self._red = new_r;
        // self._green = new_g;
        // self._blue = new_b;
    }

    pub fn get_min_max_all_channel(&self) -> (f32, f32) {
        let mut minval = std::f32::MAX;
        let mut maxval = std::f32::MIN;

        for i in 0..self.bands.len() {
            let mnmx = self.bands[i].get_min_max();
            minval = min!(mnmx.min, minval);
            maxval = max!(mnmx.max, maxval);
        }
        (minval, maxval)
    }

    pub fn normalize_between(&mut self, min: f32, max: f32) {
        for i in 0..self.bands.len() {
            self.bands[i] = self.bands[i].normalize(min, max).unwrap();
        }
    }

    pub fn normalize_to_8bit_with_max(&mut self, max: f32) {
        for i in 0..self.bands.len() {
            self.bands[i] = self.bands[i]
                .normalize_force_minmax(0.0, 255.0, 0.0, max)
                .unwrap();
            self.bands[i].clip_mut(0.0, 255.0);
        }
        self.mode = enums::ImageMode::U8BIT;
    }

    pub fn normalize_to_12bit_with_max(&mut self, max12bit: f32, max: f32) {
        for i in 0..self.bands.len() {
            self.bands[i] = self.bands[i]
                .normalize_force_minmax(0.0, max12bit, 0.0, max)
                .unwrap();
        }
        self.mode = enums::ImageMode::U12BIT;
    }

    pub fn normalize_to_16bit_with_max(&mut self, max: f32) {
        for i in 0..self.bands.len() {
            self.bands[i] = self.bands[i]
                .normalize_force_minmax(0.0, 65535.0, 0.0, max)
                .unwrap();
            self.bands[i].clip_mut(0.0, 65535.0);
        }
        self.mode = enums::ImageMode::U16BIT;
    }

    pub fn normalize_band_to_12bit(&mut self, band: usize, max12bit: f32) {
        check_band_in_bounds!(band, self);
        let mnmx = self.bands[band].get_min_max();
        self.normalize_to_12bit_with_max(max12bit, mnmx.max);
    }

    pub fn normalize_to_12bit(&mut self, max12bit: f32) {
        let (_, maxval) = self.get_min_max_all_channel();
        self.normalize_to_12bit_with_max(max12bit, maxval);
    }

    pub fn normalize_to_8bit(&mut self) {
        let (_, maxval) = self.get_min_max_all_channel();
        self.normalize_to_8bit_with_max(maxval);
    }

    pub fn normalize_to_16bit(&mut self) {
        let (_, maxval) = self.get_min_max_all_channel();
        self.normalize_to_16bit_with_max(maxval);
    }

    pub fn normalize_16bit_to_8bit(&mut self) {
        self.normalize_to_8bit_with_max(65535.0);
    }

    pub fn normalize_8bit_to_16bit(&mut self) {
        self.normalize_to_16bit_with_max(255.0);
    }

    fn save_16bit_mono(&self, to_file: &str, band: usize) {
        check_band_in_bounds!(band, self);
        let mut out_img =
            DynamicImage::new_rgba16(self.width as u32, self.height as u32).into_rgba16();

        for y in 0..self.height {
            for x in 0..self.width {
                let r = self.bands[band].get(x, y).unwrap().round() as u16;
                let a: u16 = if self.get_alpha_at(x, y) { 65535 } else { 0 };
                out_img.put_pixel(x as u32, y as u32, Rgba([r, r, r, a]));
            }
        }

        if path::parent_exists_and_writable(to_file) {
            out_img.save(to_file).unwrap();
        } else {
            panic!(
                "Parent path does not exist or is unwritable: {}",
                path::get_parent(to_file)
            );
        }
    }

    fn save_16bit_rgba(&self, to_file: &str) {
        check_band_in_bounds!(2, self);
        let mut out_img =
            DynamicImage::new_rgba16(self.width as u32, self.height as u32).into_rgba16();

        for y in 0..self.height {
            for x in 0..self.width {
                let r = self.bands[0].get(x, y).unwrap().round() as u16;
                let g = self.bands[1].get(x, y).unwrap().round() as u16;
                let b = self.bands[2].get(x, y).unwrap().round() as u16;
                let a = if self.get_alpha_at(x, y) { 65535 } else { 0 };
                out_img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
            }
        }

        if path::parent_exists_and_writable(to_file) {
            out_img.save(to_file).unwrap();
        } else {
            panic!(
                "Parent path does not exist or is unwritable: {}",
                path::get_parent(to_file)
            );
        }
    }

    fn save_16bit(&self, to_file: &str) {
        if self.bands.len() == 1 {
            self.save_16bit_mono(to_file, 0);
        } else if self.bands.len() >= 3 {
            self.save_16bit_rgba(to_file);
        } else {
            panic!("Unsupported number of bands. Cannot save as implemented");
        }
    }

    fn save_8bit_mono(&self, to_file: &str, band: usize) {
        check_band_in_bounds!(band, self);

        let mut out_img =
            DynamicImage::new_rgba8(self.width as u32, self.height as u32).into_rgba8();

        for y in 0..self.height {
            for x in 0..self.width {
                let r = self.bands[band].get(x, y).unwrap().round() as u8;
                let a = if self.get_alpha_at(x, y) { 255 } else { 0 };
                out_img.put_pixel(x as u32, y as u32, Rgba([r, r, r, a]));
            }
        }

        if path::parent_exists_and_writable(to_file) {
            out_img.save(to_file).unwrap();
        } else {
            panic!(
                "Parent path does not exist or is unwritable: {}",
                path::get_parent(to_file)
            );
        }
    }

    fn save_8bit_rgba(&self, to_file: &str) {
        check_band_in_bounds!(2, self);

        let mut out_img =
            DynamicImage::new_rgba8(self.width as u32, self.height as u32).into_rgba8();

        for y in 0..self.height {
            for x in 0..self.width {
                let r = self.bands[0].get(x, y).unwrap().round() as u8;
                let g = self.bands[1].get(x, y).unwrap().round() as u8;
                let b = self.bands[2].get(x, y).unwrap().round() as u8;
                let a = if self.get_alpha_at(x, y) { 255 } else { 0 };
                out_img.put_pixel(x as u32, y as u32, Rgba([r, g, b, a]));
            }
        }

        if path::parent_exists_and_writable(to_file) {
            out_img.save(to_file).unwrap();
        } else {
            panic!(
                "Parent path does not exist or is unwritable: {}",
                path::get_parent(to_file)
            );
        }
    }

    fn save_8bit(&self, to_file: &str) {
        if self.bands.len() == 1 {
            self.save_8bit_mono(to_file, 0);
        } else if self.bands.len() >= 3 {
            self.save_8bit_rgba(to_file);
        } else {
            panic!("Unsupported number of bands. Cannot save as implemented");
        }
    }

    pub fn save_mono(&self, to_file: &str, band: usize) {
        match self.mode {
            enums::ImageMode::U8BIT => self.save_8bit_mono(to_file, band),
            _ => self.save_16bit_mono(to_file, band),
        };
    }

    pub fn save_rgba(&self, to_file: &str) {
        match self.mode {
            enums::ImageMode::U8BIT => self.save_8bit_rgba(to_file),
            _ => self.save_16bit_rgba(to_file),
        };
    }

    pub fn save(&self, to_file: &str) {
        match self.mode {
            enums::ImageMode::U8BIT => self.save_8bit(to_file),
            _ => self.save_16bit(to_file),
        };
    }
}
