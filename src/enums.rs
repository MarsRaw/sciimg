

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum CalFileType {
    FlatField,
    InpaintMask,
    Mask
}

// Image data value range. Doesn't enforce actual
// value data types in the structs
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum ImageMode {
    U8BIT,
    U12BIT,
    U16BIT
}



impl ImageMode {

    pub fn maxvalue(mode:ImageMode) -> f32 {
        match mode {
            ImageMode::U8BIT => 255.0,
            ImageMode::U12BIT => 2033.0, // In terms of the ILT
            ImageMode::U16BIT => 65535.0
        }
    }
}


#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Axis {
    XAxis,
    YAxis,
    ZAxis
}