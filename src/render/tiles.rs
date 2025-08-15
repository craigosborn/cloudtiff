use super::{CloudTiffResult, SyncReader};
use crate::cog::Level;
use crate::raster::Raster;
use std::collections::HashMap;
use tracing::*;

pub type TileCache = HashMap<usize, Raster>;

use super::util;

pub fn get_tiles(reader: &SyncReader, level: &Level, indices: Vec<usize>) -> TileCache {
    let tile_infos = util::tile_info_from_indices(level, indices);

    // Syncronous tile reading and extraction
    tile_infos
        .into_iter()
        .filter_map(|(index, (start, end))| {
            let n = (end - start) as usize;
            let mut buf = vec![0; n];
            match reader.0.read_range_exact(start, &mut buf) {
                Ok(_) => match level.extract_tile_from_bytes(&buf) {
                    Ok(tile) => Some((index, tile)),
                    Err(e) => {
                        warn!("Failed to extract tile: {e:?}");
                        None
                    }
                },
                Err(e) => {
                    warn!("Failed to read tile bytes: {e:?}");
                    None
                }
            }
        })
        .collect()
}

pub fn get_tile(reader: &SyncReader, level: &Level, index: usize) -> CloudTiffResult<Raster> {
    let (start, end) = level.tile_byte_range(index)?;
    let n = (end - start) as usize;
    let mut buf = vec![0; n];
    let _ = reader.0.read_range_exact(start, &mut buf)?;
    let tile = level.extract_tile_from_bytes(&buf)?;
    Ok(tile)
}

#[cfg(feature = "async")]
pub use not_sync::*;
#[cfg(feature = "async")]
mod not_sync {
    use super::super::AsyncReader;
    use super::*;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    pub async fn get_tiles_async(
        reader: &AsyncReader,
        level: &Level,
        indices: Vec<usize>,
    ) -> TileCache {
        let tile_infos = util::tile_info_from_indices(level, indices);

        // Async tile reading (IO)
        let byte_results: Vec<_> = futures::future::join_all(
            tile_infos
                .into_iter()
                .map(|info| (info, reader.0.clone()))
                .map(|((index, (start, end)), reader_clone)| {
                    tokio::spawn(async move {
                        let n = (end - start) as usize;
                        let mut buf = vec![0; n];
                        reader_clone
                            .read_range_exact_async(start, &mut buf)
                            .await
                            .map(|_| (index, buf))
                    })
                }),
        )
        .await
        .into_iter()
        .filter_map(|result| match result {
            Ok(Ok(tile_bytes)) => Some(tile_bytes),
            Ok(Err(e)) => {
                warn!("Failed to get tile bytes: {e:?}");
                None
            }
            Err(e) => {
                warn!("Failed to join while getting tile bytes: {e:?}");
                None
            }
        })
        .collect();

        // Parallel tile extraction (decompression)
        //   TODO start rayon extraction without awaiting IO
        let tile_results: Vec<_> = byte_results
            .into_iter()
            .map(|(index, bytes)| (level.clone(), index, bytes))
            .collect::<Vec<_>>()
            .into_par_iter()
            .map(|(level_clone, index, bytes)| {
                level_clone
                    .extract_tile_from_bytes(&bytes)
                    .map(|tile| (index, tile))
            })
            .collect();

        let mut tile_cache: HashMap<usize, Raster> = HashMap::new(); // TODO stream rather than cache
        for result in tile_results {
            match result {
                Ok((index, tile)) => {
                    tile_cache.insert(index, tile);
                }
                Err(e) => {
                    warn!("Failed to get tile: {e:?}")
                }
            }
        }

        tile_cache
    }

    pub async fn get_tile_async(
        reader: &AsyncReader,
        level: &Level,
        index: usize,
    ) -> CloudTiffResult<Raster> {
        let (start, end) = level.tile_byte_range(index)?;
        let n = (end - start) as usize;
        let mut buf = vec![0; n];
        let _ = reader.0.read_range_exact_async(start, &mut buf).await?;
        let tile = level.extract_tile_from_bytes(&buf)?;
        Ok(tile)
    }
}
