use super::{CloudTiff, CloudTiffResult};
use std::io::{BufReader, Cursor};
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncSeek};
use tokio::sync::Mutex;

mod level;
mod render;

impl CloudTiff {
    pub async fn open_async<R: AsyncRead + AsyncSeek + Unpin>(
        source: Arc<Mutex<R>>,
    ) -> CloudTiffResult<Self> {
        let mut buffer = Vec::with_capacity(16_384); // TODO what is realistic?
        {
            let mut locked_stream = source.lock().await;
            locked_stream.read_buf(&mut buffer).await?;
        };
        let mut sync_reader = BufReader::new(Cursor::new(buffer));
        Self::open(&mut sync_reader)
    }
}