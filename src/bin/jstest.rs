// Example of using v8 to do custom pixel-by-pixel processing (as a JS function)
// on a tile
// Usage:
// `cargo run --bin jstest && eog out.png`
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::time::Instant;
use tilemachine::custom_script::CustomScript;
use tilemachine::source::open_source;
use tilemachine::utils::ImageData;
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

fn main() {
    let script = r#"
        {
            "inputs": {
                "rgb": "file:example_data/new_zealand_1_rgb.tif",
                "dsm": "file:example_data/new_zealand_1_dsm.tif"
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
    let out_data = script.execute_on_tile(&tile_coords, &open_source).unwrap();
    let duration = start.elapsed();
    println!("took {:?}", duration);
    save_tile(&out_data);
}
