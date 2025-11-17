use crate::cog::{CloudTiff, CloudTiffResult};
use crate::io::ReadRange;
use crate::projection::Projection;
use crate::{Region, UnitFloat};
use std::io::{Read, Seek};
use std::sync::Mutex;

#[cfg(feature = "async")]
use {
    crate::io::AsyncReadRange,
    std::sync::Arc,
    tokio::io::{AsyncRead, AsyncSeek},
    tokio::sync::Mutex as AsyncMutex,
};

pub mod renderer;
pub mod tiles;
pub mod util;

pub struct ReaderRequired;

pub struct SyncReader(Arc<dyn ReadRange>);

#[cfg(feature = "async")]
#[derive(Clone)]
pub struct AsyncReader(Arc<dyn AsyncReadRange>);

#[derive(Debug)]
pub struct RenderBuilder<'a, R> {
    pub cog: &'a CloudTiff,
    pub reader: R,
    pub input_projection: Projection,
    pub region: RenderRegion,
    pub resolution: (u32, u32),
}

#[derive(Debug)]
pub enum RenderRegion {
    InputCrop(Region<UnitFloat>),
    OutputRegion((u16, Region<f64>)),
    Tile((usize, usize, usize)),
}

impl CloudTiff {
    pub fn renderer(&self) -> RenderBuilder<ReaderRequired> {
        RenderBuilder {
            cog: self,
            reader: ReaderRequired,
            input_projection: self.projection.clone(),
            region: RenderRegion::InputCrop(Region::unit()),
            resolution: self.full_dimensions(),
        }
    }
}

impl<'a, S> RenderBuilder<'a, S> {
    fn set_reader<R>(self, reader: R) -> RenderBuilder<'a, R> {
        let Self {
            cog,
            reader: _,
            input_projection,
            region,
            resolution,
        } = self;
        RenderBuilder {
            cog,
            reader,
            input_projection,
            region,
            resolution,
        }
    }
}

impl<'a> RenderBuilder<'a, ReaderRequired> {
    pub fn with_reader<R: Read + Seek + 'static>(self, reader: R) -> RenderBuilder<'a, SyncReader> {
        self.set_reader(SyncReader(Arc::new(Mutex::new(reader))))
    }

    pub fn with_arc_mutex_reader<R: Read + Seek + 'static>(
        self,
        reader: Arc<Mutex<R>>,
    ) -> RenderBuilder<'a, SyncReader> {
        self.set_reader(SyncReader(reader))
    }

    pub fn with_range_reader<R: ReadRange + 'static>(
        self,
        reader: R,
    ) -> RenderBuilder<'a, SyncReader> {
        self.set_reader(SyncReader(Arc::new(reader)))
    }

    #[cfg(feature = "async")]
    pub fn with_async_reader<R: AsyncRead + AsyncSeek + Send + Sync + Unpin + 'static>(
        self,
        reader: Arc<AsyncMutex<R>>,
    ) -> RenderBuilder<'a, AsyncReader> {
        self.set_reader(AsyncReader(reader))
    }

    #[cfg(feature = "async")]
    pub fn with_async_range_reader<R: AsyncReadRange + 'static>(
        self,
        reader: R,
    ) -> RenderBuilder<'a, AsyncReader> {
        self.set_reader(AsyncReader(Arc::new(reader)))
    }

    #[cfg(feature = "async")]
    pub fn with_async_arc_range_reader<R: AsyncReadRange + 'static>(
        self,
        reader: Arc<R>,
    ) -> RenderBuilder<'a, AsyncReader> {
        self.set_reader(AsyncReader(reader))
    }
}

impl<'a, S> RenderBuilder<'a, S> {
    pub fn with_exact_resolution(mut self, resolution: (u32, u32)) -> Self {
        self.resolution = resolution;
        self
    }

    pub fn with_mp_limit(mut self, max_megapixels: f64) -> Self {
        self.resolution =
            util::resolution_from_mp_limit(self.cog.full_dimensions(), max_megapixels);
        self
    }

    pub fn of_crop(mut self, min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Self {
        self.region = RenderRegion::InputCrop(Region::new_saturated(min_x, min_y, max_x, max_y));
        self
    }

    pub fn of_output_region_lat_lon_deg(
        self,
        west: f64,
        south: f64,
        north: f64,
        east: f64,
    ) -> Self {
        self.of_output_region(
            4326,
            west.to_radians(),
            south.to_radians(),
            east.to_radians(),
            north.to_radians(), 
        )
    }

    pub fn of_output_region(
        mut self,
        epsg: u16,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
    ) -> Self {
        self.region = RenderRegion::OutputRegion((epsg, Region::new(min_x, min_y, max_x, max_y)));
        self
    }

    pub fn of_tile(
        mut self,
        x: usize,
        y: usize,
        z: usize,
    ) -> Self {
        self.region = RenderRegion::Tile((x,y,z));
        self
    }
}
