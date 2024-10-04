// https://docs.ogc.org/is/19-008r4/19-008r4.html#_requirements_class_geokeydirectorytag

use std::fmt::Display;

use super::{get_geo_tag_values, GeoKeyId, GeoKeyValue, GeoTiffError};
use crate::tiff::{Endian, Ifd, TagData, TagId, TagType};

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
    pub fn new() -> Self {
        Self {
            version: 1,
            revision: (1, 0),
            keys: vec![],
        }
    }

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
                                s[start..end]
                                    .to_string()
                                    .trim_end_matches(|c| c == '|' || c == '\0')
                                    .to_string(),
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

    pub fn add_to_ifd(&self, ifd: &mut Ifd, endian: Endian) {
        let (key_directory, ascii_params, double_params) = self.unparse();
        ifd.set_tag(
            TagId::GeoKeyDirectory,
            TagData::Short(key_directory),
            endian,
        );
        if ascii_params.len() > 0 {
            ifd.set_tag(TagId::GeoAsciiParams, TagData::Ascii(ascii_params), endian);
        }
        if double_params.len() > 0 {
            ifd.set_tag(
                TagId::GeoDoubleParams,
                TagData::Double(double_params),
                endian,
            );
        }
    }

    pub fn unparse(&self) -> (Vec<u16>, Vec<u8>, Vec<f64>) {
        let mut directory = vec![];
        let mut shorts = vec![];
        let mut asciis = vec![];
        let mut doubles = vec![];
        let dir_size = 4 * (self.keys.len() + 1) as u16;

        // Directory header
        directory.push(self.version);
        directory.push(self.revision.0);
        directory.push(self.revision.1);
        directory.push(self.keys.len() as u16);

        // Keys
        for key in &self.keys {
            directory.push(key.code);

            match &key.value {
                GeoKeyValue::Short(vec) => match vec.len() {
                    0 => directory.extend([0, 0, 0]),
                    1 => {
                        directory.push(0);
                        directory.push(1);
                        directory.push(vec[0]);
                    }
                    n => {
                        directory.push(TagId::GeoKeyDirectory as u16);
                        directory.push(n as u16);
                        directory.push(dir_size + shorts.len() as u16);
                        shorts.extend(vec);
                    }
                },
                GeoKeyValue::Ascii(s) => {
                    directory.push(TagId::GeoAsciiParams as u16);
                    directory.push(s.len() as u16);
                    directory.push(asciis.len() as u16);
                    asciis.extend(s.bytes());
                }
                GeoKeyValue::Double(vec) => {
                    directory.push(TagId::GeoDoubleParams as u16);
                    directory.push(vec.len() as u16);
                    directory.push(doubles.len() as u16);
                    doubles.extend(vec);
                }
                GeoKeyValue::Undefined => directory.extend([0, 0, 0]),
            }
        }

        ([directory, shorts].concat(), asciis, doubles)
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
