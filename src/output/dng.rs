use crate::enums::ImageMode;
use crate::image::Image;
use crate::path;
use anyhow::{anyhow, Result};
use dng::ifd::{Ifd, IfdValue};
use dng::tags::IfdType;
use dng::{tags, DngWriter, FileType};
use std::fs::File;
use std::sync::Arc;

// Note: This much code duplication is ridiculous and should not stay this way

pub fn save_image_to_rgb_16bpp(output_file_name: &str, image: &Image) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if image.num_bands() < 3 {
        return Err(anyhow!(
            "Image contains insufficient bands to produce an RGB output"
        ));
    }
    if image.get_mode() != ImageMode::U16BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }

    let file = File::create(output_file_name).unwrap();
    let mut ifd = Ifd::new(IfdType::Ifd);
    // ifd.insert(tags::ifd::Copyright, "Mars Raw Utils");
    ifd.insert(tags::ifd::ImageWidth, image.width as u32);
    ifd.insert(tags::ifd::ImageLength, image.height as u32);
    ifd.insert(tags::ifd::BitsPerSample, 16);
    ifd.insert(tags::ifd::Compression, 1); // 1 == No Compression
    ifd.insert(tags::ifd::RowsPerStrip, image.height as u32);
    ifd.insert(tags::ifd::PhotometricInterpretation, 2); // 34892 == LinearRaw  2 == RGB (RGB (Red, Green, Blue))
    ifd.insert(tags::ifd::Orientation, 1);
    ifd.insert(tags::ifd::SamplesPerPixel, 3);
    ifd.insert(tags::ifd::BlackLevel, 0);
    ifd.insert(tags::ifd::WhiteLevel, 65535);
    ifd.insert(
        tags::ifd::StripByteCounts,
        (image.width * image.height * 2 * 3) as u32,
    ); // 3 here refers to 3 color channels

    let mut band_bytes = Vec::with_capacity(image.width * image.height * 2 * 3);

    let band_values = vec![
        image.get_band(0).to_vector_u16(),
        image.get_band(1).to_vector_u16(),
        image.get_band(2).to_vector_u16(),
    ];

    (0..band_values[0].len()).for_each(|i| {
        (0..3).for_each(|b| {
            let bytes = band_values[b][i].to_le_bytes();
            band_bytes.push(bytes[0]);
            band_bytes.push(bytes[1]);
        });
    });
    ifd.insert(
        tags::ifd::StripOffsets,
        IfdValue::Offsets(Arc::new(band_bytes)),
    );

    DngWriter::write_dng(file, true, FileType::Dng, vec![ifd]).expect("Failed to write DNG file");
    Ok(())
}

pub fn save_image_to_rgba_16bpp(output_file_name: &str, image: &Image) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if !image.is_using_alpha() {
        return Err(anyhow!(
            "Image contains insufficient bands to produce an RGBA output"
        ));
    }
    if image.get_mode() != ImageMode::U16BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }

    Err(anyhow!("Not yet implemented"))
}

pub fn save_image_to_mono_16bpp(
    output_file_name: &str,
    image: &Image,
    use_band: usize,
) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if image.get_mode() != ImageMode::U16BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }
    if image.num_bands() <= use_band {
        return Err(anyhow!(
            "Image does not contain the requested color band: {}",
            use_band
        ));
    }

    let file = File::create(output_file_name).unwrap();
    let mut ifd = Ifd::new(IfdType::Ifd);
    // ifd.insert(tags::ifd::Copyright, "Mars Raw Utils");
    ifd.insert(tags::ifd::ImageWidth, image.width as u32);
    ifd.insert(tags::ifd::ImageLength, image.height as u32);
    ifd.insert(tags::ifd::BitsPerSample, 16);
    ifd.insert(tags::ifd::Compression, 1); // 1 == No Compression
    ifd.insert(tags::ifd::RowsPerStrip, image.height as u32);
    ifd.insert(tags::ifd::PhotometricInterpretation, 34892); // 34892 == LinearRaw  2 == RGB (RGB (Red, Green, Blue))
    ifd.insert(tags::ifd::Orientation, 1);
    ifd.insert(tags::ifd::SamplesPerPixel, 3);
    ifd.insert(tags::ifd::BlackLevel, 0);
    ifd.insert(tags::ifd::WhiteLevel, 65535);
    ifd.insert(
        tags::ifd::StripByteCounts,
        (image.width * image.height * 2) as u32,
    );

    let mut band_bytes = Vec::with_capacity(image.width * image.height * 2);

    let band_values = image.get_band(use_band).to_vector_u16();

    (0..band_values.len()).for_each(|i| {
        let bytes = band_values[i].to_le_bytes();
        band_bytes.push(bytes[0]);
        band_bytes.push(bytes[1]);
    });
    ifd.insert(
        tags::ifd::StripOffsets,
        IfdValue::Offsets(Arc::new(band_bytes)),
    );

    DngWriter::write_dng(file, true, FileType::Dng, vec![ifd]).expect("Failed to write DNG file");
    Ok(())
}

pub fn save_image_to_rgb_8bpp(output_file_name: &str, image: &Image) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if image.num_bands() < 3 {
        return Err(anyhow!(
            "Image contains insufficient bands to produce an RGB output"
        ));
    }
    if image.get_mode() != ImageMode::U8BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }

    let file = File::create(output_file_name).unwrap();
    let mut ifd = Ifd::new(IfdType::Ifd);
    // ifd.insert(tags::ifd::Copyright, "Mars Raw Utils");
    ifd.insert(tags::ifd::ImageWidth, image.width as u32);
    ifd.insert(tags::ifd::ImageLength, image.height as u32);
    ifd.insert(tags::ifd::BitsPerSample, 8);
    ifd.insert(tags::ifd::Compression, 1); // 1 == No Compression
    ifd.insert(tags::ifd::RowsPerStrip, image.height as u32);
    ifd.insert(tags::ifd::PhotometricInterpretation, 2); // 34892 == LinearRaw  2 == RGB (RGB (Red, Green, Blue))
    ifd.insert(tags::ifd::Orientation, 1);
    ifd.insert(tags::ifd::SamplesPerPixel, 3);
    ifd.insert(tags::ifd::BlackLevel, 0);
    ifd.insert(tags::ifd::WhiteLevel, 255);
    ifd.insert(
        tags::ifd::StripByteCounts,
        (image.width * image.height * 3) as u32,
    ); // 3 in a case like this refers to 3 color channels

    let mut band_bytes = Vec::with_capacity(image.width * image.height * 3);

    let band_values = vec![
        image.get_band(0).to_vector_u8(),
        image.get_band(1).to_vector_u8(),
        image.get_band(2).to_vector_u8(),
    ];

    (0..band_values[0].len()).for_each(|i| {
        (0..3).for_each(|b| {
            let bytes = band_values[b][i].to_le_bytes();
            band_bytes.push(bytes[0]);
        });
    });
    ifd.insert(
        tags::ifd::StripOffsets,
        IfdValue::Offsets(Arc::new(band_bytes)),
    );

    DngWriter::write_dng(file, true, FileType::Dng, vec![ifd]).expect("Failed to write DNG file");
    Ok(())
}

pub fn save_image_to_rgba_8bpp(output_file_name: &str, image: &Image) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if !image.is_using_alpha() {
        return Err(anyhow!(
            "Image contains insufficient bands to produce an RGBA output"
        ));
    }
    if image.get_mode() != ImageMode::U8BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }

    Err(anyhow!("Not yet implemented"))
}

pub fn save_image_to_mono_8bpp(
    output_file_name: &str,
    image: &Image,
    use_band: usize,
) -> Result<()> {
    if !path::parent_exists_and_writable(output_file_name) {
        return Err(anyhow!("Unable to open output path for writing: Parent path not found or permission denied. {}", output_file_name));
    }
    if image.get_mode() != ImageMode::U8BIT {
        // Rather, I just refuse to
        return Err(anyhow!("Cannot save non-16bpp data as a 16bpp image"));
    }
    if image.num_bands() <= use_band {
        return Err(anyhow!(
            "Image does not contain the requested color band: {}",
            use_band
        ));
    }

    let file = File::create(output_file_name).unwrap();
    let mut ifd = Ifd::new(IfdType::Ifd);
    // ifd.insert(tags::ifd::Copyright, "Mars Raw Utils");
    ifd.insert(tags::ifd::ImageWidth, image.width as u32);
    ifd.insert(tags::ifd::ImageLength, image.height as u32);
    ifd.insert(tags::ifd::BitsPerSample, 8);
    ifd.insert(tags::ifd::Compression, 1); // 1 == No Compression
    ifd.insert(tags::ifd::RowsPerStrip, image.height as u32);
    ifd.insert(tags::ifd::PhotometricInterpretation, 34892); // 34892 == LinearRaw  2 == RGB (RGB (Red, Green, Blue))
    ifd.insert(tags::ifd::Orientation, 1);
    ifd.insert(tags::ifd::SamplesPerPixel, 1);
    ifd.insert(tags::ifd::BlackLevel, 0);
    ifd.insert(tags::ifd::WhiteLevel, 255);
    ifd.insert(
        tags::ifd::StripByteCounts,
        (image.width * image.height) as u32,
    );

    let mut band_bytes = Vec::with_capacity(image.width * image.height);

    let band_values = image.get_band(use_band).to_vector_u8();

    (0..band_values.len()).for_each(|i| {
        let bytes = band_values[i].to_le_bytes();
        band_bytes.push(bytes[0]);
    });
    ifd.insert(
        tags::ifd::StripOffsets,
        IfdValue::Offsets(Arc::new(band_bytes)),
    );

    DngWriter::write_dng(file, true, FileType::Dng, vec![ifd]).expect("Failed to write DNG file");
    Ok(())
}
