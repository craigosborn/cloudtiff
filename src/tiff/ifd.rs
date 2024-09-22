use eio::FromBytes;

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
        stream.seek(SeekFrom::Start(offset))?;

        let tag_count = match variant {
            Variant::Normal => endian.read::<2, u16>(stream)? as u64,
            Variant::Big => endian.read(stream)?,
        };

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

    pub fn get_tag(&self, code: u16) -> Option<&Tag> {
        // TODO would a HashMap be better
        let Self(tags) = &self;
        tags.iter().find(|tag| tag.code == code)
    }

    pub fn get_required_tag(&self, id: TagId) -> Result<&Tag, TiffError> {
        self.get_tag(id.into()).ok_or(TiffError::MissingTag(id))
    }

    pub fn get_required_tag_values<const N: usize, T: FromBytes<N> + Copy>(
        &self,
        id: TagId,
    ) -> Result<Vec<T>, TiffError> {
        self.get_required_tag(id)?
            .try_raw_values()
            .ok_or(TiffError::BadTag(id))
    }

    pub fn get_required_tag_value<const N: usize, T: FromBytes<N> + Copy>(
        &self,
        id: TagId,
    ) -> Result<T, TiffError> {
        let values = self.get_required_tag_values(id)?;
        if values.len() == 1 {
            Ok(values[0])
        } else {
            Err(TiffError::BadTag(id))
        }
    }

    pub fn get_required_tag_number(&self, id: TagId) -> Result<f64, TiffError> {
        // TODO casting to and from f64 is not optimal
        self.get_required_tag(id)?
            .value()
            .into_number()
            .ok_or(TiffError::BadTag(id))
    }

    pub fn get_required_tag_vec(&self, id: TagId) -> Result<Vec<f64>, TiffError> {
        // TODO casting to and from f64 is not optimal
        self.get_required_tag(id)?
            .value()
            .into_vec()
            .ok_or(TiffError::BadTag(id))
    }

    pub fn get_tile<R: Read + Seek>(
        &self,
        stream: &mut R,
        tile_index: usize,
    ) -> Result<Tile, TiffError> {
        let compression = self.get_required_tag_value(TagId::Compression)?;
        let bits_per_sample = self.get_required_tag_values(TagId::BitsPerSample)?;
        let photometric_interpretation =
            self.get_required_tag_value(TagId::PhotometricInterpretation)?;
        let tile_width: u16 = self.get_required_tag_value(TagId::TileWidth)?;
        let tile_length: u16 = self.get_required_tag_value(TagId::TileLength)?;
        let tile_offsets: Vec<u64> = self
            .get_required_tag_vec(TagId::TileOffsets)?
            .into_iter()
            .map(|v| v as u64)
            .collect();
        let tile_byte_counts: Vec<usize> = self
            .get_required_tag_vec(TagId::TileByteCounts)?
            .into_iter()
            .map(|v| v as usize)
            .collect();

        // Validate tile_index
        let max_valid_tile_index = tile_offsets.len().min(tile_byte_counts.len()) - 1;
        if tile_index > max_valid_tile_index {
            return Err(TiffError::TileOutOfRange((
                tile_index,
                max_valid_tile_index,
            )));
        }

        let offset = tile_offsets[tile_index];
        let byte_count = tile_byte_counts[tile_index];
        let mut data = vec![0; byte_count];

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
