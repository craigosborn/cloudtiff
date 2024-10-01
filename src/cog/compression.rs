// https://en.wikipedia.org/wiki/TIFF#TIFF_Compression_Tag
// https://exiftool.org/TagNames/EXIF.html#Compression
// https://github.com/image-rs/image-tiff/blob/master/src/decoder/mod.rs

// TODO support jpeg
// TODO decide on miniz_oxide vs flate2

use std::io::{self, Read};
// use miniz_oxide::inflate::{self,TINFLStatus};
use num_enum::{FromPrimitive, IntoPrimitive};
use salzweg::decoder::{DecodingError, TiffStyleDecoder};
use flate2;

#[derive(Debug)]
pub enum DecompressError {
    LzwError(DecodingError),
    // InflateError(TINFLStatus),
    CompressionNotSupported(Compression),
    PredictorNotSupported(Predictor),
    IoError(io::Error),
}

impl From<io::Error> for DecompressError {
    fn from(e: io::Error) -> Self {
        DecompressError::IoError(e)
    }
}

#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum Compression {
    Uncompressed = 1,
    CCITT1D = 2,
    T4Group3Fax = 3,
    T6Group4Fax = 4,
    Lzw = 5,
    JpegOld = 6,
    Jpeg = 7,
    DeflateAdobe = 8,
    JbigBW = 9,
    JbigColor = 10,
    JPEGOther = 99,
    Kodak262 = 262,
    Next = 32766,
    SonyARWCompressed = 32767,
    PackedRAW = 32769,
    SamsungSRWCompressed = 32770,
    CCIRLEW = 32771,
    SamsungSRWCompressed2 = 32772,
    PackBits = 32773,
    Thunderscan = 32809,
    KodakKDCCompressed = 32867,
    IT8CTPAD = 32895,
    IT8LW = 32896,
    IT8MP = 32897,
    IT8BL = 32898,
    PixarFilm = 32908,
    PixarLog = 32909,
    Deflate = 32946,
    DCS = 32947,
    AperioJPEG2000YCbCr = 33003,
    AperioJPEG2000RGB = 33005,
    JBIG = 34661,
    SGILog = 34676,
    SGILog24 = 34677,
    JPEG2000 = 34712,
    NikonNEFCompressed = 34713,
    JBIG2TIFFFX = 34715,
    MdiBinaryLevelCodec = 34718,
    MdiProgressiveTransformCodec = 34719,
    MdiVector = 34720,
    ESRILerc = 34887,
    LossyJPEG = 34892,
    LZMA2 = 34925,
    Zstd = 34926,
    WebP = 34927,
    PNG = 34933,
    JPEGXR = 34934,
    JPEGXL = 52546,
    KodakDCRCompressed = 65000,
    PentaxPEFCompressed = 65535,

    #[num_enum(default)]
    Unknown = 0x0000,
}

impl Compression {
    pub fn decode(&self, bytes: &[u8]) -> Result<Vec<u8>, DecompressError> {
        match self {
            Self::Uncompressed => Ok(bytes.to_vec()),
            Self::Lzw => {
                TiffStyleDecoder::decode_to_vec(bytes).map_err(|e| DecompressError::LzwError(e))
            }
            Self::DeflateAdobe => {
                let mut buf = vec![];
                flate2::read::ZlibDecoder::new(bytes).read_to_end(&mut buf)?;
                Ok(buf)
                // inflate::decompress_to_vec_zlib(bytes).map_err(|e| DecompressError::InflateError(e.status))
            }
            other => Err(DecompressError::CompressionNotSupported(*other)),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, IntoPrimitive, FromPrimitive)]
#[repr(u16)]
pub enum Predictor {
    No = 1,
    Horizontal = 2,
    FloatingPoint = 3,

    #[num_enum(default)]
    Unknown = 0x0000,
}

impl Predictor {
    pub fn predict(
        &self,
        buffer: &mut [u8],
        width: usize,
        bit_depth: usize,
        samples_per_pixel: usize,
    ) -> Result<(), DecompressError> {
        match self {
            Self::No => {}
            Self::Horizontal => {
                assert!(
                    bit_depth <= 8,
                    "Bit depth {bit_depth} not supported for Horizontal Predictor"
                );
                let row_bytes = width * samples_per_pixel * bit_depth / 8;
                for i in 0..buffer.len() {
                    if i % row_bytes < samples_per_pixel {
                        continue;
                    }
                    buffer[i] = buffer[i].wrapping_add(buffer[i - samples_per_pixel]);
                }
            }
            other => return Err(DecompressError::PredictorNotSupported(*other)),
        }
        Ok(())
    }
}
