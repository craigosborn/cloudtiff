use cloudtiff::CloudTiff;
use image::DynamicImage;
use s3reader::{S3Reader, S3ObjectUri};
use std::io::BufReader;
use std::time::Instant;

const S3_PATH: &str = "s3://sentinel-cogs/sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
const OUTPUT_FILE: &str = "data/tile.tif";

fn main() {
    // File access
    println!("Opening `{S3_PATH}`");
    let uri = S3ObjectUri::new(S3_PATH).unwrap();
    let s3obj = S3Reader::open(uri).unwrap();
    let reader = &mut BufReader::new(s3obj);

    // CloudTiff indexing
    let t_cog = Instant::now();
    let cog = CloudTiff::open(reader).unwrap();
    println!("{cog}");
    println!("Indexed COG in {}us", t_cog.elapsed().as_micros());

    // Tile extraction
    let t_tile = Instant::now();
    let tile = cog.get_tile_at_lat_lon(reader, 0, 54.54890822105085, -127.78036580546008).unwrap();
    println!("Got tile in {}us", t_tile.elapsed().as_micros());
    println!("{}", tile);

    // Image output
    let img: DynamicImage = tile.try_into().unwrap();
    img.save(OUTPUT_FILE).unwrap();
    println!("Image saved to {OUTPUT_FILE}");
}