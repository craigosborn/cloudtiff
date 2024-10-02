use std::fmt::Display;
use std::io::{self, Read, Seek};

mod endian;
mod error;
mod ifd;
mod tag;

pub use endian::Endian;
pub use error::TiffError;
pub use ifd::Ifd;
pub use tag::{Tag, TagId, TagType};

#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TiffVariant {
    Normal,
    Big,
}

impl TiffVariant {
    fn read_offset<R: Read>(&self, endian: Endian, stream: &mut R) -> io::Result<u64> {
        match self {
            TiffVariant::Normal => endian.read::<4, u32>(stream).map(|v| v as u64),
            TiffVariant::Big => endian.read(stream),
        }
    }
    const fn offset_bytesize(&self) -> usize {
        match self {
            TiffVariant::Normal => 4,
            TiffVariant::Big => 8,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Tiff {
    pub endian: Endian,
    pub variant: TiffVariant,
    pub ifds: Vec<Ifd>,
}

impl Tiff {
    pub fn open<R: Read + Seek>(stream: &mut R) -> Result<Self, TiffError> {
        // TIFF Header
        let mut buf = [0; 4];
        stream.read_exact(&mut buf)?;

        let endian = match &buf[..2] {
            b"II" => Endian::Little,
            b"MM" => Endian::Big,
            _ => return Err(TiffError::BadMagicBytes),
        };

        let variant = match &buf[2..4] {
            b"\0*" | b"*\0" => TiffVariant::Normal,
            b"\0+" | b"+\0" => TiffVariant::Big,
            _ => return Err(TiffError::BadMagicBytes),
        };

        if TiffVariant::Big == variant {
            // BigTIFFs have 4 extra bytes in the header
            let _offset_bytesize: u16 = endian.read(stream)?; // 0x0008
            let _: u16 = endian.read(stream)?; // 0x0000
        }

        // IFDs
        let mut ifds = vec![];
        let mut ifd_offset = variant.read_offset(endian, stream)?;
        while ifd_offset != 0 {
            let (ifd, next_offset) = Ifd::parse(stream, ifd_offset, endian, variant)?;
            ifd_offset = next_offset;
            ifds.push(ifd);
        }

        Ok(Self {
            endian,
            variant,
            ifds,
        })
    }

    pub fn ifd0(&self) -> Result<&Ifd, TiffError> {
        if self.ifds.len() > 0 {
            Ok(&self.ifds[0])
        } else {
            Err(TiffError::NoIfd0)
        }
    }
}

impl Display for Tiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tiff: {{{:?} Endian, {:?} Variant}}",
            self.endian, self.variant
        )?;
        for (i, ifd) in self.ifds.iter().enumerate() {
            write!(f, "\n  IFD {i}:")?;
            for tag in ifd.0.iter() {
                write!(f, "\n    {}", tag)?;
            }
        }
        Ok(())
    }
}
