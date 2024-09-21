# cloudtiff

A Cloud Optimized GeoTIFF library for Rust

### Goals
* Fast
* Correct writer
* Robust reader
* Rust only
* COG focused

## Use

TBD

## Dev

### Design Principles
* Integration agnostic library. Encode and decode, don't read and write.
* Examples show integration specific usage
* Async and multithreading are optional features
* Focus on COG, don't implement the entire GeoTIFF or TIFF formats.
* No bloat, dependencies must also be focused
* Rust only dependencies

### Setup

Get sample data:
```
mkdir data
aws s3 cp --no-sign-request s3://sentinel-cogs/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif data/terrace.tif
```

### References
[TIFF 6.0 spec](https://download.osgeo.org/geotiff/spec/tiff6.pdf)
[OGC GeoTIFF standard](https://docs.ogc.org/is/19-008r4/19-008r4.html)  
[GeoTIFF paper](https://www.geospatialworld.net/wp-content/uploads/images/pdf/117.pdf)
[Cloud Optimized GeoTIFF spec](https://github.com/cogeotiff/cog-spec/blob/master/spec.md)  
[COG spec article](https://cogeotiff.github.io/rio-cogeo/Is_it_a_COG/)
[COG introduction article](https://developers.planet.com/docs/planetschool/an-introduction-to-cloud-optimized-geotiffs-cogs-part-1-overview/)
[COG use article](https://medium.com/@_VincentS_/do-you-really-want-people-using-your-data-ec94cd94dc3f)
[COG on AWS article](https://opengislab.com/blog/2021/4/17/hosting-and-accessing-cloud-optimized-geotiffs-on-aws-s3)  

### Sample Data
[AWS Sentinel-2](https://registry.opendata.aws/sentinel-2-l2a-cogs/)  
[NASA EarthData](https://www.earthdata.nasa.gov/engage/cloud-optimized-geotiffs)  
[rio-tiler](https://github.com/cogeotiff/rio-tiler/tree/6.4.0/tests/fixtures)  

### Related Libraries
[cog3pio](https://github.com/weiji14/cog3pio) (Read only)  
[tiff](https://crates.io/crates/tiff)  
[geo](https://crates.io/crates/geo) (Coordinate transformation and projection)  
[geotiff](https://crates.io/crates/geotiff)   
[rio-tiler](https://github.com/cogeotiff/rio-tiler) (Python)  
[rio-cogeo](https://github.com/cogeotiff/rio-cogeo) (Python)  
[gdal](https://crates.io/crates/gdal) (Rust bindings for GDAL)  

### Tools
[QGIS](https://cogeo.org/qgis-tutorial.html)  
[GDAL](https://gdal.org/en/latest/drivers/raster/cog.html)  