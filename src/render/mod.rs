use crate::cog::{CloudTiff, CloudTiffResult};
use crate::projection::Projection;
use crate::{AsyncReadRange, ReadRange, Region, UnitFloat};

mod not_sync;
mod renderer;
mod sync;
pub mod tiles;
pub mod util;

pub use sync::SyncRender;

#[cfg(feature = "async")]
pub use not_sync::AsyncRender;

// Lifetime of RenderBuilder:
// borrows cog until RenderBuilder is consumed during render
// therefore, renderbuilder should not normally be kept as a long term variable
// instead, use RenderBuilder to configure a render immediately prior to rendering
#[derive(Debug)]
pub struct RenderBuilder<'c> {
    pub cog: &'c CloudTiff,
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
    pub fn renderer<'c>(&'c self) -> RenderBuilder<'c> {
        RenderBuilder {
            cog: self,
            input_projection: self.projection.clone(),
            region: RenderRegion::InputCrop(Region::unit()),
            resolution: self.full_dimensions(),
        }
    }
}

impl<'c> RenderBuilder<'c> {
    pub fn with_reader<'r, R: ReadRange>(self, reader: &'r R) -> SyncRender<'c, 'r, R> {
        SyncRender::new(self, reader)
    }

    #[cfg(feature = "async")]
    pub fn with_async_reader<R: AsyncReadRange + 'static>(self, reader: R) -> AsyncRender<'c, R> {
        AsyncRender::new(self, reader)
    }

    // pub fn with_arc_mutex_reader<R: Read + Seek + 'static>(
    //     self,
    //     reader: Arc<Mutex<R>>,
    // ) -> RenderBuilder<SyncReader> {
    //     self.set_reader(SyncReader(reader))
    // }

    // pub fn with_range_reader<R: ReadRange + 'static>(self, reader: R) -> RenderBuilder<SyncReader> {
    //     self.set_reader(SyncReader(Arc::new(reader)))
    // }

    // #[cfg(feature = "async")]
    // pub fn with_async_reader<R: AsyncRead + AsyncSeek + Send + Sync + Unpin + 'static>(
    //     self,
    //     reader: Arc<AsyncMutex<R>>,
    // ) -> RenderBuilder<AsyncReader> {
    //     self.set_reader(AsyncReader(reader))
    // }

    // #[cfg(feature = "async")]
    // pub fn with_async_range_reader<R: AsyncReadRange + 'static>(
    //     self,
    //     reader: R,
    // ) -> RenderBuilder<AsyncReader> {
    //     self.set_reader(AsyncReader(Arc::new(reader)))
    // }

    // #[cfg(feature = "async")]
    // pub fn with_async_arc_range_reader<R: AsyncReadRange + 'static>(
    //     self,
    //     reader: Arc<R>,
    // ) -> RenderBuilder<AsyncReader> {
    //     self.set_reader(AsyncReader(reader))
    // }
}

impl<'c> RenderBuilder<'c> {
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
        east: f64,
        north: f64,
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

    pub fn of_tile(mut self, x: usize, y: usize, z: usize) -> Self {
        self.region = RenderRegion::Tile((x, y, z));
        self
    }
}
