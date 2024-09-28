use crate::cog::{CloudTiffError, CloudTiffResult, Level};
use crate::raster::Raster;
use futures::future::join_all;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek, AsyncSeekExt, SeekFrom};
use tokio::sync::Mutex;
use tokio::task::JoinError;

impl Level {
    pub async fn stream_tiles_in_region<'a, R: AsyncRead + AsyncSeek + Unpin + Send + 'static>(
        &self,
        source: Arc<Mutex<R>>,
        region: (f64, f64, f64, f64),
    ) -> Vec<Result<CloudTiffResult<(Raster, usize, (f64, f64, f64, f64))>, JoinError>> {
        let indices = self.tile_indices_within_image_region(region);
        self.stream_tiles(source, indices).await
    }

    pub async fn stream_tiles<'a, R: AsyncRead + AsyncSeek + Unpin + Send + 'static>(
        &self,
        reader: Arc<Mutex<R>>,
        indices: Vec<usize>, // (left, top, right, bottom)
    ) -> Vec<Result<CloudTiffResult<(Raster, usize, (f64, f64, f64, f64))>, JoinError>> {
        let self_arc = Arc::new(self.clone()); // TODO is this the best way?

        let byte_results = join_all(
            indices
                .into_iter()
                .map(|index| (self_arc.clone(), reader.clone(), index))
                .map(|(self_clone, reader_clone, index)| {
                    tokio::spawn(async move {
                        self_clone
                            .get_tile_bytes_async(reader_clone, index)
                            .await
                            .map(|bytes| (self_clone, bytes, index))
                    })
                }),
        )
        .await;

        join_all(byte_results.into_iter().map(|result| {
            tokio::spawn(async move {
                match result {
                    Ok(Ok((self_clone, mut bytes, index))) => self_clone
                        .extract_tile_bytes(&mut bytes)
                        .map(|r| (r, index, self_clone.tile_bounds(&index))),
                    Ok(Err(e)) => Err(e),
                    Err(_e) => Err(CloudTiffError::NoLevels),
                }
            })
        })).await
    }

    pub async fn get_tile_at_image_coords_async<R: AsyncRead + AsyncSeek + Unpin>(
        &self,
        reader: Arc<Mutex<R>>,
        x: f64,
        y: f64,
    ) -> Result<Raster, CloudTiffError> {
        let (index, _tile_x, _tile_y) = self.index_from_image_coords(x, y)?;
        self.get_tile_by_index_async(reader, index).await
    }

    pub async fn get_tile_by_index_async<R: AsyncRead + AsyncSeek + Unpin>(
        &self,
        reader: Arc<Mutex<R>>,
        index: usize,
    ) -> CloudTiffResult<Raster> {
        let mut bytes = self.get_tile_bytes_async(reader.clone(), index).await?;
        self.extract_tile_bytes(&mut bytes)
    }

    async fn get_tile_bytes_async<R: AsyncRead + AsyncSeek + Unpin>(
        &self,
        reader: Arc<Mutex<R>>,
        index: usize,
    ) -> CloudTiffResult<Vec<u8>> {
        // Validate index
        let max_valid_index = self.offsets.len().min(self.byte_counts.len()) - 1;
        if index > max_valid_index {
            return Err(CloudTiffError::TileIndexOutOfRange((
                index,
                max_valid_index,
            )));
        }

        // Lookup byte range
        let offset = self.offsets[index];
        let byte_count = self.byte_counts[index];

        // Read bytes
        let mut bytes = vec![0; byte_count];
        {
            let mut locked_reader = reader.lock().await;
            locked_reader.seek(SeekFrom::Start(offset)).await?;
            locked_reader.read_exact(&mut bytes).await?;

        }; // Lock is released here


        Ok(bytes)
    }
}
