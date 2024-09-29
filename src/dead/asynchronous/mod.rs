use super::{CloudTiff, CloudTiffResult};
use std::io::Cursor;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::Mutex;

mod level;
mod render;

impl CloudTiff {
    pub async fn open_async<R: AsyncRead + Unpin>(source: Arc<Mutex<R>>) -> CloudTiffResult<Self> {
        let mut buffer = vec![0; 16_384]; // TODO what is realistic?
        let mut locked_source = source.lock().await;
        locked_source.read(&mut buffer).await?;
        let mut sync_reader = Cursor::new(buffer);
        Self::open(&mut sync_reader)
    }
}
