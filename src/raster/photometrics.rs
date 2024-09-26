use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum PhotometricInterpretation {
    WhiteIsZero = 0,
    BlackIsZero = 1,
    RGB = 2,
    RGBPalette = 3,
    TransparencyMask = 4,
    CMYK = 5,
    YCbCr = 6,
    CIELab = 8,
    ICCLab = 9,
    ITULab = 10,
    ColorFilterArray = 32803,
    PixarLogL = 32844,
    PixarLogLuv = 32845,
    SequentialColorFilter = 32892,
    LinearRaw = 34892,
    DepthMap = 51177,
    SemanticMask = 52527,

    #[num_enum(default)]
    Unknown = 0xFFFF,
}

#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum SampleFormat {
    Uint = 1,
    Int = 2,
    IEEEFP = 3,
    Undefined = 4,

    #[num_enum(default)]
    Unknown = 0xFFFF,
}

#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum PlanarConfiguration {
    Chunky = 1,
    Planar = 2,

    #[num_enum(default)]
    Unknown = 0xFFFF,
}