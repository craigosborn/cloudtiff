use num_traits::NumCast;

use super::{Endian, Tag, TagId, TagType, TiffError, Tile, Variant};
use std::io::{self, Read, Seek, SeekFrom};

#[derive(Clone, Debug)]
pub struct Ifd(pub Vec<Tag>);

impl Ifd {
    pub fn parse<R: Read + Seek>(
        stream: &mut R,
        offset: u64,
        endian: Endian,
        variant: Variant,
    ) -> io::Result<(Ifd, u64)> {
        // IFD starts at offset
        stream.seek(SeekFrom::Start(offset))?;

        // IFD header is just the number of tags
        let tag_count = match variant {
            Variant::Normal => endian.read::<2, u16>(stream)? as u64,
            Variant::Big => endian.read(stream)?,
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
        let code = id.into();
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

    pub fn get_tile<R: Read + Seek>(
        &self,
        stream: &mut R,
        tile_index: usize,
    ) -> Result<Tile, TiffError> {
        // Required tags
        let compression = self.get_tag_value(TagId::Compression)?;
        let bits_per_sample = self.get_tag_values(TagId::BitsPerSample)?;
        let photometric_interpretation = self.get_tag_value(TagId::PhotometricInterpretation)?;
        let tile_width = self.get_tag_value(TagId::TileWidth)?;
        let tile_length = self.get_tag_value(TagId::TileLength)?;
        let tile_offsets = self.get_tag_values(TagId::TileOffsets)?;
        let byte_counts = self.get_tag_values(TagId::TileByteCounts)?;

        // Validate tile_index
        let max_valid_tile_index = tile_offsets.len().min(byte_counts.len()) - 1;
        if tile_index > max_valid_tile_index {
            return Err(TiffError::TileOutOfRange((
                tile_index,
                max_valid_tile_index,
            )));
        }

        // Indexed tile
        let offset = tile_offsets[tile_index];
        let byte_count = byte_counts[tile_index];
        let mut data = vec![0; byte_count];

        // Tile bytes
        stream.seek(SeekFrom::Start(offset))?;
        stream.read_exact(&mut data)?;

        Ok(Tile {
            width: tile_width,
            height: tile_length,
            compression,
            bits_per_sample,
            photometric_interpretation,
            data,
        })
    }
}
