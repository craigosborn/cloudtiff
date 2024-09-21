use super::tag::{Tag, TagId, TagType};
use super::Endian;
use super::Variant;
use std::io::{Read, Result, Seek, SeekFrom};

#[derive(Clone, Debug)]
pub struct Ifd(pub Vec<Tag>);

impl Ifd {
    pub fn parse<R: Read + Seek>(
        stream: &mut R,
        offset: u64,
        endian: Endian,
        variant: Variant,
    ) -> Result<(Ifd, u64)> {
        stream.seek(SeekFrom::Start(offset))?;

        let tag_count = match variant {
            Variant::Normal => endian.read::<2, u16>(stream)? as u64,
            Variant::Big => endian.read(stream)?,
        };

        let mut tags = Vec::with_capacity(tag_count as usize);
        for _ in 0..tag_count {
            let id: TagId = endian.read::<2, u16>(stream)?.into();
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
                id,
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
}
