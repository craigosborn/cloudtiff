use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use cloudtiff::CloudTiff;
use image::DynamicImage;
use std::f64::consts::{PI, TAU};
use std::fs::File;
use std::io::BufReader;
use std::sync::{Arc, Mutex};

const SAMPLE_COG: &str = "data/sample.tif";
const TILE_SIZE: u32 = 256;
const HOST_URL: &str = "localhost:8080";

struct AppState {
    pub reader: Arc<Mutex<BufReader<File>>>,
    pub cog: CloudTiff,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // cog
    let file = File::open(SAMPLE_COG).unwrap();
    let mut reader = BufReader::new(file);
    let cog = CloudTiff::open(&mut reader).unwrap();

    // state
    let state = AppState {
        cog,
        reader: Arc::new(Mutex::new(reader)),
    };
    let app_state = web::Data::new(state);

    // server
    println!("WMTS example is running at http://{HOST_URL} (Ctrl+C to stop)");
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .route("/", web::get().to(index))
            .route("/bounds", web::get().to(get_bounds))
            .route("/tiles/{z}/{x}/{y}.png", web::get().to(get_tile))
    })
    .bind(HOST_URL)? // Bind to localhost on port 8080
    .run()
    .await
}

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(HTML_WEBMAP)
}

async fn get_bounds(state: web::Data<AppState>) -> impl Responder {
    let (north, west, south, east) = state.cog.bounds_lat_lon_deg().unwrap();

    let json_bounds = format!("[[{south}, {west}], [{north}, {east}]]");
    HttpResponse::Ok()
        .content_type("application/json")
        .body(json_bounds)
}

async fn get_tile(
    state: web::Data<AppState>,
    path: web::Path<(usize, usize, usize)>,
) -> impl Responder {
    let (z, x, y) = path.into_inner();

    let tile_bounds = tile_bounds_lat_lon_deg(x, y, z).unwrap();
    match state
        .cog
        .render_region_lat_lon_deg_async(
            state.reader.clone(),
            tile_bounds,
            (TILE_SIZE, TILE_SIZE),
        )
        .await
    {
        Ok(tile) => {
            let img: DynamicImage = tile.try_into().unwrap();
            let mut png_bytes: Vec<u8> = Vec::new();
            img.write_to(
                &mut std::io::Cursor::new(&mut png_bytes),
                image::ImageFormat::Png,
            )
            .unwrap();
            HttpResponse::Ok().content_type("image/png").body(png_bytes)
        }
        Err(_e) => HttpResponse::NotFound().body("Tile not available."),
    }
}

fn tile_bounds_lat_lon_deg(x: usize, y: usize, z: usize) -> Option<(f64, f64, f64, f64)> {
    let (north, west) = tile_index_to_lat_lon_deg(x as f64, y as f64, z as f64)?;
    let (south, east) = tile_index_to_lat_lon_deg((x + 1) as f64, (y + 1) as f64, z as f64)?;
    Some((north, west, south, east))
}

fn tile_index_to_lat_lon_deg(x: f64, y: f64, z: f64) -> Option<(f64, f64)> {
    let n = 2.0_f64.powf(z);
    if x < 0.0 || x / n > 1.0 || y < 0.0 || y / n > 1.0 || z < 0.0 {
        return None;
    }
    let lon = x * TAU / n - PI;
    let var = PI * (1.0 - 2.0 * y / n);
    let lat = (0.5 * ((var).exp() - (-var).exp())).atan();
    Some((lat.to_degrees(), lon.to_degrees()))
}

const HTML_WEBMAP: &str = r#"
<!DOCTYPE html>
<html>
<head>
    <title>WMTS Example</title>
    <link rel="stylesheet" href="https://unpkg.com/leaflet/dist/leaflet.css" />
    <script src="https://unpkg.com/leaflet/dist/leaflet.js"></script>
    <style>
        body {margin: 0;}
        #map {height: 100vh;}
        #opacity-slider {
            display: flex;
            position: absolute;
            bottom: 0px;
            left: 0px;
            width: 50%;
            z-index: 1000;
            background: white;
            padding: 10px;
            border-radius: 5px;
            opacity: 70%;
        }
        #opacity{flex: 1;}
    </style>
</head>
<body>
    <div id="map"></div>
     <div id="opacity-slider">
        <label for="opacity">Opacity: </label>
        <input type="range" id="opacity" min="0" max="1" step="0.1" value="1">
    </div>
    <script>
        // Leaflet map
        var map = L.map('map').setView([0, 0], 2);

        // OSM basemap layer
        L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
            attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors'
        }).addTo(map);
    
        // Connect to our example WMTS server
        var cogLayer;
        fetch('./bounds')
            .then(response=>response.json())
            .then(bounds => {
                // Zoom to COG
                map.fitBounds(bounds);

                // Add COG as WMTS layer
                cogLayer = L.tileLayer('./tiles/{z}/{x}/{y}.png', {
                    layer: 'cog'
                }).addTo(map);
            })
            .catch(error => console.error('Error fetching bounds:', error));

        // Opacity
        document.getElementById('opacity').addEventListener('input', function() {
            cogLayer && cogLayer.setOpacity(this.value);
        });
    </script>
</body>
</html>
"#;
