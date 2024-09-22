use std::fmt::Display;

pub struct Tile {
    pub width: u16,
    pub height: u16,
    pub compression: u16,
    pub bits_per_sample: Vec<u16>,
    pub photometric_interpretation: u16,
    pub data: Vec<u8>,
}

impl Display for Tile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Tile: {{width: {}, height: {}, compression: {}, bps: {:?}, pi: {}, data: {}bytes}}",
            self.width,
            self.height,
            self.compression,
            self.bits_per_sample,
            self.photometric_interpretation,
            self.data.len()
        )
    }
}
