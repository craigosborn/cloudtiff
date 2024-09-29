// use cloudtiff::CloudTiff;
// use image::DynamicImage;
use object_store::{aws::AmazonS3Builder, ObjectStore, path::Path};
// use std::time::Instant;
// use aws_sdk_s3::{primitives::ByteStream,config::Region, Client};
use tokio;

// https://docs.rs/object_store/0.11.0/object_store/
// https://crates.io/crates/aws-sdk-s3

const BUCKET_NAME: &str = "sentinel-cogs";
// const OBJECT_NAME: &str = "sentinel-s2-l2a-cogs/9/U/WA/2024/8/S2A_9UWA_20240806_0_L2A/TCI.tif";
// const OUTPUT_FILE: &str = "data/tile.tif";

fn main() {
    // File access
    println!("Opening `{BUCKET_NAME}`");

    let s3 = AmazonS3Builder::new()
        .with_region("us-west-2") // You need to provide a region even for anonymous access
        .with_bucket_name(BUCKET_NAME) // Replace with the public bucket name
        .with_allow_http(false) // If you're accessing over HTTP instead of HTTPS
        .build().unwrap();
    let path = Path::from("path/to/your/public/object.txt");

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let h  = s3.head(&path).await;
            println!("{h:?}");
            // let config =
            //     aws_config::load_defaults(aws_config::BehaviorVersion::v2024_03_28()).await;
            // let config = aws_sdk_s3::config::Builder::new()
            //     .region(Some(Region::from_static("us-west-2")))
            //     .build();
            // let config = aws_config::from_env().load().await;

            // let client = Client::new(&config);
            // let objects = client
            //     .list_objects_v2()
            //     .bucket(BUCKET_NAME)
            //     .prefix(OBJECT_NAME)
            //     .send()
            //     .await
            //     .unwrap();
            // println!("{objects:?}");
        })
    // // CloudTiff indexing
    // let t_cog = Instant::now();
    // let cog = CloudTiff::open(reader).unwrap();
    // println!("{cog}");
    // println!("Indexed COG in {}us", t_cog.elapsed().as_micros());

    // // Tile extraction
    // let t_tile = Instant::now();
    // let tile = cog
    //     .get_tile_at_lat_lon(reader, 0, 54.54890822105085, -127.78036580546008)
    //     .unwrap();
    // println!("Got tile in {}us", t_tile.elapsed().as_micros());
    // println!("{}", tile);

    // // Image output
    // let img: DynamicImage = tile.try_into().unwrap();
    // img.save(OUTPUT_FILE).unwrap();
    // println!("Image saved to {OUTPUT_FILE}");
}
