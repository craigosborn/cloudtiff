use crate::cog::{CloudTiff, CloudTiffResult};
use crate::io::ReaderFlavor;
use crate::raster::Raster;
use crate::CloudTiffError;
use std::collections::HashMap;
use std::io::{self, SeekFrom};

pub fn render_image_region(
    cog: &CloudTiff,
    flavor: ReaderFlavor,
    region: (f64, f64, f64, f64),
    dimensions: (u32, u32),
) -> CloudTiffResult<Raster> {
    // Determine render layer
    let (left, top, right, bottom) = region;
    let min_level_dims = (
        ((dimensions.0 as f64) / (right - left)).ceil() as u32,
        ((dimensions.1 as f64) / (top - bottom)).ceil() as u32,
    );
    let level = cog
        .levels
        .iter()
        .rev()
        .find(|level| {
            level.dimensions.0 > min_level_dims.0 && level.dimensions.1 > min_level_dims.1
        })
        .unwrap_or(&cog.levels[0]);

    // Output Raster
    let mut render_raster = Raster::blank(
        dimensions.clone(),
        level.bits_per_sample.clone(),
        level.interpretation,
        level.endian,
    );

    // Tiles
    let mut tile_cache: HashMap<usize, Raster> = HashMap::new(); // TODO stream rather than cache
    let indices = level.tile_indices_within_image_region(region);

    let tile_results: Vec<_> = indices
        .into_iter()
        .map(|index| match level.tile_byte_range(index) {
            Ok((start, end)) => match match &flavor {
                ReaderFlavor::ReadRange(reader) => reader.read_range(start, end),
                ReaderFlavor::ReadSeek(reader) => match reader.lock() {
                    Ok(mut locked_reader) => match locked_reader.seek(SeekFrom::Start(start)) {
                        Ok(_) => {
                            let mut buffer = vec![0; (end - start) as usize];
                            match locked_reader.read(&mut buffer) {
                                Ok(_) => Ok(buffer),
                                Err(e) => Err(e),
                            }
                        }
                        Err(e) => Err(e),
                    },
                    Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("{e:?}"))),
                },
            } {
                Ok(bytes) => level
                    .extract_tile_from_bytes(&bytes)
                    .map(|tile| (index, tile)),
                Err(e) => Err(CloudTiffError::from(e)),
            },
            Err(e) => Err(e),
        })
        .collect();

    // while let Some(result) = join_set.join_next() {
    for result in tile_results {
        match result {
            Ok((index, tile)) => {
                tile_cache.insert(index, tile);
            }
            Err(e) => {
                println!("Failed to get tile: {e:?}")
            }
        }
    }

    // Render
    let dxdi = (bottom - top) / dimensions.1 as f64;
    let mut y = top;
    let dydj = (bottom - top) / dimensions.1 as f64;
    for j in 0..dimensions.1 {
        let mut x = top;
        for i in 0..dimensions.0 {
            let (tile_index, u, v) = level.index_from_image_coords(x, y)?; // TODO cache tiles
            if let Some(tile) = tile_cache.get(&tile_index) {
                if let Some(pixel) = tile.get_pixel(u as u32, v as u32) {
                    let _ = render_raster.put_pixel(i, j, pixel);
                }
            }
            x += dxdi;
        }
        y += dydj;
    }

    Ok(render_raster)
}
