use super::projection::{Projection, UnitRegion};
use super::{CloudTiff, CloudTiffResult};
use crate::endian::Endian;
use crate::io::{Flavor, ReadRange, ReadRangeAsync};
use crate::raster::PhotometricInterpretation;
use crate::raster::Raster;
use std::marker::PhantomData;

mod old;

pub struct BuilderReady;
pub struct BuilderNotReady;

#[derive(Debug)]
pub struct RenderBuilder<'a, T, BuilderReady> {
    source: &'a T,
    reader: Option<Flavor>,
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
            input_projection: self.projection.clone(),
            input_region: UnitRegion::default(),
            output_projection: self.projection.clone(),
            output_region: UnitRegion::default(),
            output_resolution: (1, 1),
            _builder_status: PhantomData,
        }
    }
}

impl<'a, S> RenderBuilder<'a, CloudTiff, S> {
    pub fn with_reader<R: ReadRange>(
        mut self,
        reader: R,
    ) -> RenderBuilder<'a, CloudTiff, BuilderReady> {
        self.reader = Some(Flavor::Sync(Box::new(reader)));
        self.into_ready()
    }

    pub fn with_async_reader<R: ReadRangeAsync>(
        mut self,
        reader: R,
    ) -> RenderBuilder<'a, CloudTiff, BuilderReady> {
        self.reader = Some(Flavor::Async(Box::new(reader)));
        self.into_ready()
    }

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

    fn into_ready(self) -> RenderBuilder<'a, CloudTiff, BuilderReady> {
        let Self {
            source,
            reader,
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
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
            _builder_status: PhantomData,
        }
    }
}

impl<'a> RenderBuilder<'a, CloudTiff, BuilderReady> {
    pub fn render(self) -> CloudTiffResult<Raster> {
        Raster::blank(
            self.output_resolution,
            vec![8, 8, 8],
            PhotometricInterpretation::RGB,
            Endian::Little,
        );

        match self.reader {
            Some(Flavor::Sync(reader)) => old::render_image_region(
                self.source,
                reader,
                self.output_region.as_f64(),
                self.output_resolution,
            ),
            _ => Err(super::CloudTiffError::NoLevels),
        }
    }
}
