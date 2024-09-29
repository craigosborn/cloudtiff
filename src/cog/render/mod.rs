use super::CloudTiff;
use super::projection::{Projection, UnitRegion};
use crate::io::{Flavor, ReadRange, ReadRangeAsync};
use std::marker::PhantomData;

pub struct BuilderReady;
pub struct BuilderNotReady;

#[derive(Debug)]
pub struct RenderBuilder<BuilderReady> {
    reader: Option<Flavor>,
    input_projection: Projection,
    input_region: UnitRegion,
    output_projection: Projection,
    output_region: UnitRegion,
    output_resolution: (u32, u32),
    _builder_status: PhantomData<BuilderReady>,
}

pub trait Renderer {
    fn renderer(&self) -> RenderBuilder<BuilderNotReady>;
}

impl Renderer for CloudTiff {
    fn renderer(&self) -> RenderBuilder<BuilderNotReady> {
        RenderBuilder {
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

impl<S> RenderBuilder<S> {
    pub fn with_reader<R: ReadRange>(mut self, reader: R) -> RenderBuilder<BuilderReady> {
        self.reader = Some(Flavor::Sync(Box::new(reader)));
        self.into_ready()
    }

    pub fn with_async_reader<R: ReadRangeAsync>(
        mut self,
        reader: R,
    ) -> RenderBuilder<BuilderReady> {
        self.reader = Some(Flavor::Async(Box::new(reader)));
        self.into_ready()
    }

    pub fn with_exact_resolution(mut self, resolution: (u32, u32)) -> Self {
        self.output_resolution = resolution;
        self
    }

    fn into_ready(self) -> RenderBuilder<BuilderReady> {
        let Self {
            reader,
            input_projection,
            input_region,
            output_projection,
            output_region,
            output_resolution,
            _builder_status,
        } = self;
        RenderBuilder {
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

impl RenderBuilder<BuilderReady> {
    pub fn render(self) {
        todo!()
    }
}
