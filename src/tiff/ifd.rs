use num_traits::NumCast;

use super::{Endian, Tag, TagData, TagId, TagType, TiffError, TiffOffsets, TiffVariant};
use std::{
    collections::HashMap,
    io::{self, Read, Seek, SeekFrom, Write},
};

#[derive(Clone, Debug)]
pub struct Ifd(pub Vec<Tag>);

impl Ifd {
    pub fn new() -> Self {
        Self(vec![])
    }

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

    pub fn set_tag_by_code(&self, code: u16) -> Option<&Tag> {
        let Self(tags) = &self;
        tags.iter().find(|tag| tag.code == code)
    }

    pub fn set_tag<I: Into<u16>>(&mut self, id: I, data: TagData, endian: Endian) {
        // TODO hashmap tags
        let code: u16 = id.into();
        let tag = Tag::new(code, endian, data);
        let tags = &mut self.0;
        if let Some(index) = tags.iter().position(|tag| tag.code == code) {
            tags[index] = tag;
        } else {
            tags.push(tag);
        }
    }

    pub fn encode<W: Write + Seek>(
        &self,
        stream: &mut W,
        endian: Endian,
        variant: TiffVariant,
        last_ifd: bool,
    ) -> Result<TiffOffsets, io::Error> {
        // IFD header is just the number of tags
        let tag_count = self.0.len();
        match variant {
            TiffVariant::Normal => endian.write(stream, tag_count as u16)?,
            TiffVariant::Big => endian.write(stream, tag_count as u64)?,
        };

        // Things to remember
        let mut offsets = HashMap::new();
        let mut extra_data = vec![];
        let offset_size = variant.offset_bytesize();
        let (header_size, tag_size) = match variant {
            TiffVariant::Normal => (2, 12),
            TiffVariant::Big => (8, 20),
        };
        let extra_data_offset =
            stream.stream_position()? + tag_size * tag_count as u64 + offset_size as u64;

        // Write each tag in the IFD
        for (i, tag) in self.0.iter().enumerate() {
            endian.write(stream, tag.code as u16)?;
            endian.write(stream, tag.datatype as u16)?;
            variant.write_offset(endian, stream, tag.count as u64)?;

            let offset = if tag.data.len() > offset_size {
                let data_offset = extra_data_offset + extra_data.len() as u64;
                variant.write_offset(endian, stream, data_offset)?;
                extra_data.extend_from_slice(&tag.data);
                data_offset
            } else {
                let bytes: Vec<u8> = tag
                    .data
                    .clone()
                    .into_iter()
                    .chain(vec![0; offset_size].into_iter())
                    .take(offset_size)
                    .collect();
                stream.write_all(&bytes)?;
                header_size + tag_size * i as u64 + 4 + offset_size as u64
            };

            offsets.insert(tag.code, offset);
        }

        if last_ifd {
            variant.write_offset(endian, stream, 0)?;
        } else {
            let current_pos = stream.stream_position()?;
            variant.write_offset(
                endian,
                stream,
                current_pos + extra_data.len() as u64 + offset_size as u64,
            )?;
        }

        stream.write_all(&extra_data)?;

        Ok(offsets)
    }
}
