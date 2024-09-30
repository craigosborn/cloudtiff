use super::projection::{Projection, UnitRegion};
use super::{CloudTiff, CloudTiffResult};
use crate::io::{
    AsyncReadRange, AsyncReadSeek, AsyncReaderFlavor, ReadRange, ReadSeek, ReaderFlavor,
};
use crate::raster::Raster;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use tokio::sync::Mutex as AsyncMutex;

mod nold;
mod old;

pub struct BuilderNotReady;
pub struct BuilderReady;
pub struct BuilderReadyAsync;

#[derive(Debug)]
pub struct RenderBuilder<'a, T, BuilderReady> {
    source: &'a T,
    reader: Option<ReaderFlavor>,
    async_reader: Option<AsyncReaderFlavor>,
    input_projection: Projection,
    input_region: UnitRegion,
    output_projection: Projection,
    output_region: UnitRegion,
    output_resolution: (u32, u32),
    _builder_status: PhantomData<BuilderReady>,
}

impl CloudTiff {
    pub fn renderer(&self) -> RenderBuilder<CloudTiff, BuilderNotReady> {
        RenderBuilder {
            source: self,
            reader: None,
            async_reader: None,
            input_projection: self.projection.clone(),
            input_region: UnitRegion::default(),
            output_projection: self.projection.clone(),
            output_region: UnitRegion::default(),
            output_resolution: (1, 1),
            _builder_status: PhantomData,
        }
    }
}

impl<'a> RenderBuilder<'a, CloudTiff, BuilderNotReady> {
    pub fn with_reader<R: ReadSeek>(
        mut self,
        reader: R,
    ) -> RenderBuilder<'a, CloudTiff, BuilderReady> {
        self.reader = Some(ReaderFlavor::ReadSeek(Arc::new(Mutex::new(reader))));
        self.into_ready()
    }

    pub fn with_range_reader<R: ReadRange>(
        mut self,
        reader: R,
    ) -> RenderBuilder<'a, CloudTiff, BuilderReady> {
        self.reader = Some(ReaderFlavor::ReadRange(Arc::new(reader)));
        self.into_ready()
    }

    pub fn with_async_reader<R: AsyncReadSeek>(
        mut self,
        reader: R,
    ) -> RenderBuilder<'a, CloudTiff, BuilderReadyAsync> {
        self.async_reader = Some(AsyncReaderFlavor::AsyncReadSeek(Arc::new(AsyncMutex::new(
            reader,
        ))));
        self.into_ready_async()
    }

    pub fn with_async_range_reader<R: AsyncReadRange>(
        mut self,
        reader: R,
    ) -> RenderBuilder<'a, CloudTiff, BuilderReadyAsync> {
        self.async_reader = Some(AsyncReaderFlavor::AsyncReadRange(Arc::new(reader)));
        self.into_ready_async()
    }

    fn into_ready(self) -> RenderBuilder<'a, CloudTiff, BuilderReady> {
        let Self {
            source,
            reader,
            async_reader,
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
            _builder_status,
        } = self;
        RenderBuilder {
            source,
            reader,
            async_reader,
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
            _builder_status: PhantomData,
        }
    }

    fn into_ready_async(self) -> RenderBuilder<'a, CloudTiff, BuilderReadyAsync> {
        let Self {
            source,
            reader,
            async_reader,
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
            _builder_status,
        } = self;
        RenderBuilder {
            source,
            reader,
            async_reader,
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
            _builder_status: PhantomData,
        }
    }
}

impl<'a, S> RenderBuilder<'a, CloudTiff, S> {
    pub fn with_exact_resolution(mut self, resolution: (u32, u32)) -> Self {
        self.output_resolution = resolution;
        self
    }

    pub fn with_mp_limit(mut self, max_megapixels: f64) -> Self {
        let ar = self.source.aspect_ratio();
        let mp = max_megapixels.min(self.source.full_megapixels());
        let height = (mp * 1e6 / ar).sqrt();
        let width = ar * height;
        self.output_resolution = (width as u32, height as u32);
        self
    }
}

impl<'a> RenderBuilder<'a, CloudTiff, BuilderReady> {
    pub fn render(self) -> CloudTiffResult<Raster> {
        if let Some(flavor) = self.reader {
            old::render_image_region(
                self.source,
                flavor,
                self.output_region.as_f64(),
                self.output_resolution,
            )
        } else {
            panic!("Render should not be callable without Some(reader)");
        }
    }
}

impl<'a> RenderBuilder<'a, CloudTiff, BuilderReadyAsync> {
    pub async fn render_async(self) -> CloudTiffResult<Raster> {
        if let Some(async_flavor) = self.async_reader {
            nold::ender_image_region_async(
                self.source,
                async_flavor,
                self.output_region.as_f64(),
                self.output_resolution,
            )
            .await
        } else {
            panic!("Render should not be callable without Some(async_reader)");
        }
    }
}
