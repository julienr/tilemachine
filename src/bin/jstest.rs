// Example of using v8 to do custom pixel-by-pixel processing (as a JS function)
// on a tile
// Usage:
// `cargo run --bin jstest && eog out.png`
use gdal::Dataset;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::time::Instant;
use tilemachine::custom_script::CustomScript;
use tilemachine::jsengine;
use tilemachine::jsengine::ImageData;
use tilemachine::utils::Result;
use tilemachine::xyz::TileCoords;

fn save_tile(image_data: &ImageData<u8>) {
    let mut out_buf = Vec::new();
    {
        let w = BufWriter::new(&mut out_buf);
        let mut encoder = png::Encoder::new(w, image_data.width as u32, image_data.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&image_data.data).unwrap();
    }

    let mut f = File::create("out.png").unwrap();
    f.write_all(out_buf.as_slice()).unwrap();
}

fn open_dataset(filename: &str) -> Result<Dataset> {
    let ds = Dataset::open(filename)?;
    Ok(ds)
}

fn main() {
    let mut engine = jsengine::JSEngine::default();

    let script = r#"
        {
            "inputs": {
                "rgb": "example_data/raster1.tif",
                "dsm": "example_data/raster1_fake_dsm_cog.tif"
            },
            "script": "return [3 * dsm[0], rgb[1], rgb[2], 255]"
        }
    "#;

    let script = CustomScript::new_from_str(script).unwrap();
    let tile_coords = TileCoords {
        x: 175402,
        y: 410750,
        zoom: 20,
    };
    let start = Instant::now();
    let out_data = script
        .execute_on_tile(&tile_coords, &open_dataset, &mut engine)
        .unwrap();
    let duration = start.elapsed();
    println!("took {:?}", duration);
    save_tile(&out_data);
}
