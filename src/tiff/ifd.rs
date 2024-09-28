use num_traits::NumCast;

use super::{Endian, Tag, TagId, TagType, TiffError, TiffVariant};
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Clone, Debug)]
pub struct Ifd(pub Vec<Tag>);

impl Ifd {
    pub fn parse<R: Read + Seek>(
        stream: &mut R,
        offset: u64,
        endian: Endian,
        variant: TiffVariant,
    ) -> io::Result<(Ifd, u64)> {
        // IFD starts at offset
        stream.seek(SeekFrom::Start(offset))?;

        // IFD header is just the number of tags
        let tag_count = match variant {
            TiffVariant::Normal => endian.read::<2, u16>(stream)? as u64,
            TiffVariant::Big => endian.read(stream)?,
        };

        // Parse each tag in the IFD
        let mut tags = Vec::with_capacity(tag_count as usize);
        for _ in 0..tag_count {
            let code = endian.read(stream)?;
            let datatype: TagType = endian.read::<2, u16>(stream)?.into();
            let count = variant.read_offset(endian, stream)? as usize;

            let data_size = count * datatype.size_in_bytes();
            let offset_size = variant.offset_bytesize();
            let mut data: Vec<u8> = vec![0; data_size.max(offset_size)];

            if data_size > offset_size {
                let data_offset = variant.read_offset(endian, stream)? as u64;
                let pos = stream.stream_position()?;
                stream.seek(SeekFrom::Start(data_offset))?;
                stream.read_exact(&mut data)?;
                stream.seek(SeekFrom::Start(pos))?;
            } else {
                stream.read_exact(&mut data)?;
                if data_size < offset_size {
                    data = data[0..data_size].to_vec();
                }
            }

            tags.push(Tag {
                code,
                datatype,
                endian,
                count,
                data,
            });
        }

        let ifd = Ifd(tags);
        let next_ifd_offset = variant.read_offset(endian, stream)? as u64;

        Ok((ifd, next_ifd_offset))
    }

    pub fn get_tag_by_code(&self, code: u16) -> Option<&Tag> {
        let Self(tags) = &self;
        tags.iter().find(|tag| tag.code == code)
    }

    pub fn get_tag(&self, id: TagId) -> Result<&Tag, TiffError> {
        let code: u16 = id.into();
        let Self(tags) = &self;
        tags.iter()
            .find(|tag| tag.code == code)
            .ok_or(TiffError::MissingTag(id))
    }

    pub fn get_tag_values<T: NumCast>(&self, id: TagId) -> Result<Vec<T>, TiffError> {
        self.get_tag(id)?.values().ok_or(TiffError::BadTag(id))
    }

    pub fn get_tag_value<T: NumCast + Copy>(&self, id: TagId) -> Result<T, TiffError> {
        self.get_tag(id)?.value().ok_or(TiffError::BadTag(id))
    }
}
