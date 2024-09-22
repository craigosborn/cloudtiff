use std::fmt::Display;

use super::{get_geo_tag_values, GeoKeyId, GeoKeyValue, GeoTiffError};
use crate::tiff::{Ifd, TagId, TagType};

#[derive(Clone, Debug)]
pub struct GeoKeyDirectory {
    pub version: u16,
    pub revision: (u16, u16),
    pub keys: Vec<GeoKey>,
}

#[derive(Clone, Debug)]

pub struct GeoKey {
    pub code: u16,
    pub value: GeoKeyValue,
}

impl GeoKey {
    pub fn id(&self) -> Option<GeoKeyId> {
        GeoKeyId::try_from(self.code).ok()
    }
}

impl GeoKeyDirectory {
    pub fn parse(ifd: &Ifd) -> Result<Self, GeoTiffError> {
        // Directory is a tiff tag
        let directory_values = get_geo_tag_values(ifd, TagId::GeoKeyDirectory)?;

        // Directory size validation
        if directory_values.len() < 4 {
            return Err(GeoTiffError::BadTag(TagId::GeoKeyDirectory));
        }

        // Directory header
        let version: u16 = directory_values[0];
        let revision: u16 = directory_values[1];
        let minor_revision: u16 = directory_values[2];
        let key_count: u16 = directory_values[3];

        // Directory size validation
        let min_valid_directory_size = 4 + key_count * 4;
        if directory_values.len() < min_valid_directory_size as usize {
            return Err(GeoTiffError::BadTag(TagId::GeoKeyDirectory));
        }

        // Parse keys
        let keys: Vec<GeoKey> = (0..key_count as usize)
            .map(|i| {
                let entry_offset = (i + 1) * 4;
                let code = directory_values[entry_offset + 0];
                let location = directory_values[entry_offset + 1];
                let count = directory_values[entry_offset + 2];
                let offset = directory_values[entry_offset + 3];

                let value = if location == 0 {
                    GeoKeyValue::Short(vec![offset])
                } else {
                    let start = offset as usize;
                    let end = (offset + count) as usize;
                    let tag = ifd.get_tag_by_code(location);
                    tag.and_then(|tag| match tag.datatype {
                        TagType::Ascii => tag.try_to_string().map(|s| {
                            GeoKeyValue::Ascii(
                                s.trim_end_matches(|c| c == '|' || c == '\0').to_string(),
                            )
                        }),
                        TagType::Short => tag
                            .values()
                            .map(|v| GeoKeyValue::Short(v[start..end].to_vec())),
                        TagType::Double => tag
                            .values()
                            .map(|v| GeoKeyValue::Double(v[start..end].to_vec())),
                        _ => None,
                    })
                    .unwrap_or(GeoKeyValue::Undefined)
                };

                GeoKey { code, value }
            })
            .collect();

        Ok(Self {
            version,
            revision: (revision, minor_revision),
            keys,
        })
    }
}

impl Display for GeoKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let id_string = match self.id() {
            Some(id) => format!("{id:?}"),
            None => format!("0x{:04X}", self.code),
        };
        write!(f, "{}: {}", id_string, self.value)
    }
}
