use crate::geotags::{GeoKeyId, GeoModel, GeoModelScaled, GeoModelTransformed, GeoTags};
use primatives::{Point2D, Region};
use proj4rs::errors::Error as Proj4Error;
use proj4rs::proj::Proj;
use proj4rs::transform::transform;

pub mod primatives;

// COG Projection
//   TODO verify 3D support
//   TODO recognize units (e.g. degrees vs radians)

// BUG projection to 4326 seems to end up in radians

#[derive(Debug)]
pub enum ProjectionError {
    MissingGeoKey(GeoKeyId),
    Proj4Error(Proj4Error),
    InvalidOrigin((f64, f64, f64)),
    InvalidScale((f64, f64, f64)),
    UnsupportedModelTransformation,
}

impl From<Proj4Error> for ProjectionError {
    fn from(e: Proj4Error) -> Self {
        ProjectionError::Proj4Error(e)
    }
}
#[derive(Clone, Debug)]
pub struct Projection {
    pub epsg: u16,
    pub proj: Proj,
    pub origin: (f64, f64, f64),
    pub scale: (f64, f64, f64),
}

impl Projection {
    pub fn from_geo_tags(geo: &GeoTags, dimensions: (u32, u32)) -> Result<Self, ProjectionError> {
        let Some(epsg) = geo
            .directory
            .keys
            .iter()
            .find(|key| {
                matches!(
                    key.id(),
                    Some(GeoKeyId::ProjectedCSTypeGeoKey | GeoKeyId::GeographicTypeGeoKey)
                )
            })
            .and_then(|key| key.value.as_number())
        else {
            return Err(ProjectionError::MissingGeoKey(
                GeoKeyId::ProjectedCSTypeGeoKey,
            ));
        };
        let proj = Proj::from_epsg_code(epsg)?;
        // let units = proj.units();

        // TODO there has to be a better way...
        let unit_gain = match (
            epsg,
            geo.directory
                .keys
                .iter()
                .find(|key| matches!(key.id(), Some(GeoKeyId::GeogAngularUnitsGeoKey)))
                .and_then(|key| key.value.as_number()),
        ) {
            (4326, Some(9102)) => 1_f64.to_radians(),
            (4326, None) => 1_f64.to_radians(),
            _ => 1.0,
        };

        let (tiepoint, pixel_scale) = match geo.model {
            GeoModel::Transformed(GeoModelTransformed {
                transformation: _,
                tiepoint: _,
            }) => return Err(ProjectionError::UnsupportedModelTransformation), // TODO
            GeoModel::Scaled(GeoModelScaled {
                tiepoint,
                pixel_scale,
            }) => (tiepoint, pixel_scale),
        };

        let origin = (
            tiepoint[3] * unit_gain,
            tiepoint[4] * unit_gain,
            tiepoint[5] * unit_gain,
        );
        if !origin.0.is_finite() || !origin.1.is_finite() || !origin.2.is_finite() {
            return Err(ProjectionError::InvalidOrigin(origin));
        }

        let pixel_scale = (
            pixel_scale[0] * unit_gain,
            pixel_scale[1] * unit_gain,
            pixel_scale[2] * unit_gain,
        );
        if !pixel_scale.0.is_normal() || !pixel_scale.1.is_normal() {
            return Err(ProjectionError::InvalidScale(pixel_scale));
        }
        let scale = (
            pixel_scale.0 * dimensions.0 as f64,
            pixel_scale.1 * dimensions.1 as f64,
            pixel_scale.2, // TODO verify how z scale works
        );

        Ok(Self {
            epsg,
            proj,
            origin,
            scale,
        })
    }

    pub fn transform_from_lat_lon_deg(
        &self,
        lat: f64,
        lon: f64,
    ) -> Result<(f64, f64), ProjectionError> {
        let (x, y, _) = self.transform_from(lon.to_radians(), lat.to_radians(), 0.0, 4326)?;
        Ok((x, y))
    }

    pub fn transform_into_lat_lon_deg(
        &self,
        x: f64,
        y: f64,
    ) -> Result<(f64, f64), ProjectionError> {
        let (lon, lat, _) = self.transform_from(x, y, 0.0, 4326)?;
        Ok((lat.to_degrees(), lon.to_degrees()))
    }

    pub fn transform_from(
        &self,
        x: f64,
        y: f64,
        z: f64,
        epsg: u16,
    ) -> Result<(f64, f64, f64), ProjectionError> {
        let mut point = (x, y, z);
        let from = Proj::from_epsg_code(epsg)?;
        transform(&from, &self.proj, &mut point)?;

        let u = (point.0 - self.origin.0) / self.scale.0;
        let v = (self.origin.1 - point.1) / self.scale.1;
        let w = point.2 - self.origin.2; // TODO verify this calc

        Ok((u, v, w))
    }

    pub fn transform_from_proj(
        &self,
        from: &Proj,
        x: f64,
        y: f64,
        z: f64,
    ) -> Result<(f64, f64, f64), ProjectionError> {
        let mut point = (x, y, z);
        transform(from, &self.proj, &mut point)?;

        let u = (point.0 - self.origin.0) / self.scale.0;
        let v = (self.origin.1 - point.1) / self.scale.1;
        let w = point.2 - self.origin.2; // TODO verify this calc

        Ok((u, v, w))
    }

    pub fn transform_into(
        &self,
        u: f64,
        v: f64,
        w: f64,
        epsg: u16,
    ) -> Result<(f64, f64, f64), ProjectionError> {
        let x = self.origin.0 + u * self.scale.0;
        let y = self.origin.1 - v * self.scale.1;
        let z = self.origin.2 + w; // TODO verify this calc

        let mut point = (x, y, z);
        let to = Proj::from_epsg_code(epsg)?;
        transform(&self.proj, &to, &mut point)?;
        Ok(point)
    }

    pub fn transform_into_proj(
        &self,
        to: &Proj,
        u: f64,
        v: f64,
        w: f64,
    ) -> Result<(f64, f64, f64), ProjectionError> {
        let x = self.origin.0 + u * self.scale.0;
        let y = self.origin.1 - v * self.scale.1;
        let z = self.origin.2 + w; // TODO verify this calc

        let mut point = (x, y, z);
        transform(&self.proj, &to, &mut point)?;
        Ok(point)
    }

    pub fn bounds_lat_lon_deg(&self) -> Result<Region<f64>, ProjectionError> {
        let radians = self.bounds(4326);
        Ok(Region::new(
            radians.x.min.to_degrees(),
            radians.y.min.to_degrees(),
            radians.x.max.to_degrees(),
            radians.y.max.to_degrees(),
        ))
    }

    pub fn bounds(&self, epsg: u16) -> Region<f64> {
        vec![
            [0.0, 0.0],
            [0.5, 0.0],
            [1.0, 0.0],
            [1.0, 0.5],
            [1.0, 1.0],
            [0.5, 1.0],
            [0.0, 1.0],
            [0.0, 0.5],
        ]
        .into_iter()
        .fold(
            Region::new(f64::MAX, f64::MAX, f64::MIN, f64::MIN),
            |region, [u, v]| {
                if let Ok((x, y, _)) = self.transform_into(u, v, 0.0, epsg) {
                    region.extend(&Point2D { x, y })
                } else {
                    region
                }
            },
        )
    }

    pub fn bounds_in_proj(&self, proj: &Proj) -> Result<Region<f64>, ProjectionError> {
        let (left, top, _) = self.transform_into_proj(&proj, 0.0, 0.0, 0.0)?;
        let (right, bottom, _) = self.transform_into_proj(&proj, 1.0, 1.0, 0.0)?;
        Ok(Region::new(left, bottom, right, top))
    }
}
