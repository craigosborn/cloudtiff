use std::io::{Read, Seek};
use std::sync::Mutex;

use crate::cog::projection::{Projection, UnitRegion};
use crate::cog::{CloudTiff, CloudTiffResult};
use crate::io::ReadRange;

#[cfg(feature = "async")]
use {
    crate::io::AsyncReadRange,
    std::sync::Arc,
    tokio::io::{AsyncRead, AsyncSeek},
    tokio::sync::Mutex as AsyncMutex,
};

mod renderer;
mod tiles;
mod util;

pub struct ReaderRequired;

pub struct SyncReader(Box<dyn ReadRange>);

#[cfg(feature = "async")]
#[derive(Clone)]
pub struct AsyncReader(Arc<dyn AsyncReadRange>);

#[derive(Debug)]
pub struct RenderBuilder<'a, R> {
    pub cog: &'a CloudTiff,
    pub reader: R,
    pub input_projection: Projection,
    pub input_region: UnitRegion,
    pub output_projection: Projection,
    pub output_region: UnitRegion,
    pub output_resolution: (u32, u32),
}

impl CloudTiff {
    pub fn renderer(&self) -> RenderBuilder<ReaderRequired> {
        RenderBuilder {
            cog: self,
            reader: ReaderRequired,
            input_projection: self.projection.clone(),
            input_region: UnitRegion::default(),
            output_projection: self.projection.clone(),
            output_region: UnitRegion::default(),
            output_resolution: (1, 1),
        }
    }
}

impl<'a> RenderBuilder<'a, ReaderRequired> {
    pub fn with_reader<R: Read + Seek + 'static>(self, reader: R) -> RenderBuilder<'a, SyncReader> {
        let Self {
            cog,
            reader: _,
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
        } = self;
        RenderBuilder {
            cog,
            reader: SyncReader(Box::new(Mutex::new(reader))),
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
        }
    }

    pub fn with_range_reader<R: ReadRange + 'static>(
        self,
        reader: R,
    ) -> RenderBuilder<'a, SyncReader> {
        let Self {
            cog,
            reader: _,
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
        } = self;
        RenderBuilder {
            cog,
            reader: SyncReader(Box::new(reader)),
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
        }
    }

    #[cfg(feature = "async")]
    pub fn with_async_reader<R: AsyncRead + AsyncSeek + Send + Sync + Unpin + 'static>(
        self,
        reader: Arc<AsyncMutex<R>>,
    ) -> RenderBuilder<'a, AsyncReader> {
        let Self {
            cog,
            reader: _,
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
        } = self;
        RenderBuilder {
            cog,
            reader: AsyncReader(reader),
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
        }
    }

    #[cfg(feature = "async")]
    pub fn with_async_range_reader<R: AsyncReadRange + 'static>(
        self,
        reader: R,
    ) -> RenderBuilder<'a, AsyncReader> {
        let Self {
            cog,
            reader: _,
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
        } = self;
        RenderBuilder {
            cog,
            reader: AsyncReader(Arc::new(reader)),
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
        }
    }
}

impl<'a, S> RenderBuilder<'a, S> {
    pub fn with_exact_resolution(mut self, resolution: (u32, u32)) -> Self {
        self.output_resolution = resolution;
        self
    }

    pub fn with_mp_limit(mut self, max_megapixels: f64) -> Self {
        self.output_resolution =
            util::resolution_from_mp_limit(self.cog.full_dimensions(), max_megapixels);
        self
    }

    pub fn with_image_region(mut self, region: (f64, f64, f64, f64)) -> Self {
        let (min_x, min_y, max_x, max_y) = region;
        self.input_region = UnitRegion::new(min_x, min_y, max_x, max_y).unwrap(); // TODO
        self
    }
}
