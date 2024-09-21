// https://docs.ogc.org/is/19-008r4/19-008r4.html#_geotiff_tags_for_coordinate_transformations

use num_enum::{FromPrimitive, IntoPrimitive};

#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum TagId {
    SubfileType = 0x00FE,
    ImageWidth = 0x0100,
    ImageHeight = 0x0101,
    BitsPerSample = 0x0102,
    Compression = 0x0103,
    PhotometricInterpretation = 0x0106,
    SamplesPerPixel = 0x0115,
    RowsPerStrip = 0x0116,
    StripByteCounts = 0x0117,
    MinSampleValue = 0x0118,
    MaxSampleValue = 0x0119,
    XResolution = 0x011A,
    YResolution = 0x011B,
    PlanarConfiguration = 0x011C,
    ResolutionUnit = 0x0128,
    Predictor = 0x013D,
    ColorMap = 0x0140,
    TileWidth = 0x0142,
    TileLength = 0x0143,
    TileOffsets = 0x0144,
    TileByteCounts = 0x0145,
    SampleFormat = 0x0153,
    ModelPixelScale = 0x830E,
    ModelTiepoint = 0x8482,
    ModelTransformation = 0x85D8,
    GeoKeyDirectory = 0x87AF,
    GeoDoubleParams = 0x87B0,
    GeoAsciiParams = 0x87B1,
    GDALMetadata = 0xA480,
    GDALNoData = 0xA481,

    #[num_enum(default)]
    Unknown = 0xFFFF,
}
