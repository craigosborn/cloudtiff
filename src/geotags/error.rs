use crate::tiff::TagId;

#[derive(Debug)]
pub enum GeoTiffError {
    MissingTag(TagId),
    BadTag(TagId),
}
