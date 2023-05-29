pub mod dng;
pub mod standard;
use crate::enums::ImageMode;
use crate::image::Image;
use anyhow::{anyhow, Result};
use std::env;

use std::ffi::OsStr;
use std::path::Path;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum OutputFormat {
    PNG,
    JPEG,
    TIFF,
    DNG,
}

impl OutputFormat {
    pub fn from_string(s: &str) -> Result<OutputFormat> {
        match s.to_uppercase().as_str() {
            "PNG" => Ok(OutputFormat::PNG),
            "JPG" | "JPEG" => Ok(OutputFormat::JPEG),
            "TIF" | "TIFF" => Ok(OutputFormat::TIFF),
            "DNG" => Ok(OutputFormat::DNG),
            _ => Err(anyhow!("Invalid output format specified: {}", s)),
        }
    }
}

pub fn get_default_output_format() -> Result<OutputFormat> {
    if let Ok(output_format_str) = env::var("MARS_OUTPUT_FORMAT") {
        OutputFormat::from_string(&output_format_str)
    } else {
        Ok(OutputFormat::PNG)
    }
}

pub fn determine_output_format_from_filename(filename: &str) -> Result<OutputFormat> {
    if let Some(extension) = Path::new(filename).extension().and_then(OsStr::to_str) {
        match extension.to_string().to_uppercase().as_str() {
            "DNG" => Ok(OutputFormat::DNG),
            "PNG" => Ok(OutputFormat::PNG),
            "TIF" | "TIFF" => Ok(OutputFormat::TIFF),
            "JPG" | "JPEG" => Ok(OutputFormat::JPEG),
            _ => Err(anyhow!(
                "Unable to determine output format or is an unsupported format: {}",
                extension
            )),
        }
    } else {
        Err(anyhow!("Unable to isolate filename extension"))
    }
}

pub fn replace_extension_with(filename: &str, new_extension: &str) -> Result<String> {
    if let Some(new_filename) = Path::new(filename).with_extension(new_extension).to_str() {
        Ok(new_filename.to_string())
    } else {
        Err(anyhow!("Unable to replace filename"))
    }
}

pub fn replace_extension_for(filename: &str, format: OutputFormat) -> Result<String> {
    match format {
        OutputFormat::DNG => replace_extension_with(filename, "dng"),
        OutputFormat::PNG => replace_extension_with(filename, "png"),
        OutputFormat::TIFF => replace_extension_with(filename, "tif"),
        OutputFormat::JPEG => replace_extension_with(filename, "jpg"),
    }
}

/// Saves an image to the specified path. The format (png, tiff, dng, etc) is determined from the
/// filename extension.
pub fn save_image_to(output_file_name: &str, image: &Image) -> Result<()> {
    match determine_output_format_from_filename(output_file_name) {
        Ok(format) => save_image_with_format(output_file_name, format, image),
        Err(why) => Err(why),
    }
}

/// Saves an image to the specified path. Forces an output format
pub fn save_image_with_format(
    output_file_name: &str,
    format: OutputFormat,
    image: &Image,
) -> Result<()> {
    if image.num_bands() == 1 {
        save_image_mono_using_band_with_format(output_file_name, format, image, 0)
    } else {
        let corrected_output_file_name =
            if let Ok(filename) = replace_extension_for(output_file_name, format) {
                filename
            } else {
                return Err(anyhow!("Unexpected error with filename"));
            };

        match image.get_mode() {
            ImageMode::U8BIT => {
                if format == OutputFormat::DNG && image.is_using_alpha() {
                    dng::save_image_to_rgba_8bpp(&corrected_output_file_name, image)
                } else if format == OutputFormat::DNG {
                    dng::save_image_to_rgb_8bpp(&corrected_output_file_name, image)
                } else if image.is_using_alpha() {
                    // The others all use the same functions.
                    standard::save_image_to_rgba_8bpp(&corrected_output_file_name, image)
                } else {
                    standard::save_image_to_rgb_8bpp(&corrected_output_file_name, image)
                }
            }
            ImageMode::U12BIT => Err(anyhow!("12 bit output not yet implemented")),
            ImageMode::U16BIT => {
                if format == OutputFormat::DNG && image.is_using_alpha() {
                    dng::save_image_to_rgba_16bpp(&corrected_output_file_name, image)
                } else if format == OutputFormat::DNG {
                    dng::save_image_to_rgb_16bpp(&corrected_output_file_name, image)
                } else if image.is_using_alpha() {
                    standard::save_image_to_rgba_16bpp(&corrected_output_file_name, image)
                } else {
                    // The others all use the same functions.
                    standard::save_image_to_rgb_16bpp(&corrected_output_file_name, image)
                }
            }
        }
    }
}

pub fn save_image_mono_using_band(
    output_file_name: &str,
    image: &Image,
    use_band: usize,
) -> Result<()> {
    match determine_output_format_from_filename(output_file_name) {
        Ok(format) => {
            save_image_mono_using_band_with_format(output_file_name, format, image, use_band)
        }
        Err(why) => Err(why),
    }
}

pub fn save_image_mono_using_band_with_format(
    output_file_name: &str,
    format: OutputFormat,
    image: &Image,
    use_band: usize,
) -> Result<()> {
    let corrected_output_file_name =
        if let Ok(filename) = replace_extension_for(output_file_name, format) {
            filename
        } else {
            return Err(anyhow!("Unexpected error with filename"));
        };

    match image.get_mode() {
        ImageMode::U8BIT => {
            if format == OutputFormat::DNG {
                dng::save_image_to_mono_8bpp(&corrected_output_file_name, image, use_band)
            } else {
                // The others all use the same functions.
                standard::save_image_to_mono_8bpp(&corrected_output_file_name, image, use_band)
            }
        }
        ImageMode::U12BIT => Err(anyhow!("12 bit output not yet implemented")),
        ImageMode::U16BIT => {
            if format == OutputFormat::DNG {
                dng::save_image_to_mono_16bpp(&corrected_output_file_name, image, use_band)
            } else {
                // The others all use the same functions.
                standard::save_image_to_mono_16bpp(&corrected_output_file_name, image, use_band)
            }
        }
    }
}
