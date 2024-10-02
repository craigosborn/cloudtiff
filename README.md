# cloudtiff

A Cloud Optimized GeoTIFF library for Rust

### Goals

* COG focused
* Fast
* Robust reader
* Correct writer
* Pure Rust

### Features

- [x] TIFF decoding without extracting 
- [x] Tile extraction and decompression
- [x] Georeferencing from tags (proj4rs)
- [x] Tile rerendering (WMTS)
- [ ] Encoding
- [x] Integration with S3 & HTTP

### Limitations

* Predictor only supports None or Horizontal 8bit
* Decompression only supports None or Lzw or Deflate


## Use

```rs
use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::fs::File;
use std::io::BufReader;

fn save_preview(file: File) {
    let reader = &mut BufReader::new(file);
    let cog = CloudTiff::open(reader).unwrap();

    let preview = cog.render_image_with_mp_limit(reader, 1.0).unwrap();

    let img: DynamicImage = preview.try_into().unwrap();
    img.save("preview.jpg").unwrap();
}
```

## Dev

### Setup

Get sample data:
```
mkdir data
aws s3 cp --no-sign-request s3://sentinel-cogs/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif data/sample.tif
```

Run the example:
```
cargo run --example wmts
```

### Design Principle
* Integration agnostic library. Encode and decode, don't read and write.
* Examples show integration specific usage
* Async and multithreading are optional features
* Focus on COG, don't implement the entire GeoTIFF or TIFF formats.
* No bloat, dependencies must also be focused
* Rust only dependencies

### References
[TIFF 6.0 spec](https://download.osgeo.org/geotiff/spec/tiff6.pdf)  
[BigTIFF spec](https://web.archive.org/web/20240622111852/https://www.awaresystems.be/imaging/tiff/bigtiff.html)  
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
[tiff](https://crates.io/crates/tiff) (Decoding not optimal for COG)  
[geo](https://crates.io/crates/geo) (Coordinate transformation and projection)  
[geotiff](https://crates.io/crates/geotiff) (Decoding not optimal for COG)  
[geotiff-rs](https://github.com/fizyk20/geotiff-rs)  
[gdal](https://crates.io/crates/gdal) (Rust bindings for GDAL)  

### Tools
[QGIS](https://cogeo.org/qgis-tutorial.html)  
[GDAL](https://gdal.org/en/latest/drivers/raster/cog.html)  
[rio-cogeo](https://github.com/cogeotiff/rio-cogeo)  
[rio-tiler](https://github.com/cogeotiff/rio-tiler)  