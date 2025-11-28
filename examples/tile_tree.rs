#![cfg(feature = "image")]

use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

const INPUT_COG: &str = "data/sample.tif";
const OUTPUT_TEMPLATE: &str = "data/tiles/{z}/{y}/{x}.png";

// Use
// cargo run --example tile -- path/to/some/cog.tif output/{z}/tile_{y}_{x}.tif

fn main() {
    println!("Example: cloudtiff tile");
    let t0 = Instant::now();

    // Arguments
    let args: Vec<String> = env::args().collect();
    let input_cog = if args.len() > 1 {
        args[1].clone()
    } else {
        String::from(INPUT_COG)
    };
    let output_template = if args.len() > 2 {
        args[2].parse().unwrap()
    } else {
        String::from(OUTPUT_TEMPLATE)
    };

    let mut file = fs::File::open(input_cog).unwrap();
    let cog = CloudTiff::open(&mut file).unwrap();

    let n_tiles = render_tiles(&cog, file, output_template);

    println!(
        "Saved {} tiles in {:.3}ms",
        n_tiles,
        t0.elapsed().as_micros() as f64 / 1000.0
    );
}

fn render_tiles(cog: &CloudTiff, file: fs::File, output_template: String) -> usize {
    // optional on unix
    let file = std::sync::Mutex::new(file);

    let mut n_tiles = 0;
    for (z, level) in cog.levels.iter().enumerate() {
        let template_z = output_template.replace("{z}", &z.to_string());

        for y in 0..level.col_count() {
            let template_y = template_z.replace("{y}", &y.to_string());
            if let Some(parent) = PathBuf::from(&template_y).parent() {
                let _ = fs::create_dir_all(parent);
            }

            for x in 0..level.col_count() {
                let tile_path = template_y.replace("{x}", &x.to_string());

                let tile = cog
                    .renderer()
                    .of_tile(x, y, z)
                    .with_reader(&file)
                    .render()
                    .unwrap();

                let img: DynamicImage = tile.try_into().unwrap();
                img.save(tile_path).unwrap();
                n_tiles += 1;
            }
        }
    }
    n_tiles
}
