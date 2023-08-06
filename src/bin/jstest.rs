// Example of using v8 to do custom pixel-by-pixel processing (as a JS function)
// on a tile
// Usage:
// `cargo run --bin jstest && eog out.png`
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::time::Instant;
use tilemachine::jsengine;

fn load_tile() -> (Vec<u8>, usize, usize) {
    let decoder = png::Decoder::new(File::open("example_data/tile.png").unwrap());
    let mut reader = decoder.read_info().unwrap();
    let height = reader.info().height;
    let width = reader.info().width;
    if reader.info().color_type != png::ColorType::Rgb {
        panic!("Expected RGB color type");
    }
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).unwrap();
    let bytes = &buf[..info.buffer_size()];
    (bytes.into(), width as usize, height as usize)
}

fn save_tile(bytes: &[u8], size: (usize, usize)) {
    let mut out_buf = Vec::new();
    {
        let w = BufWriter::new(&mut out_buf);
        let mut encoder = png::Encoder::new(w, size.0 as u32, size.1 as u32);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(bytes).unwrap();
    }

    let mut f = File::create("out.png").unwrap();
    f.write_all(out_buf.as_slice()).unwrap();
}

fn main() {
    let mut engine = jsengine::JSEngine::default();
    let (mut tile_data, width, height) = load_tile();

    let start = Instant::now();
    let code = "return [r * 2, g, b]";
    engine.exec(code, &mut |compiled_func| {
        for i in 0..height {
            for j in 0..width {
                let rgb = &mut tile_data[i * width * 3 + j * 3..i * width * 3 + (j + 1) * 3];
                let output_rgb = compiled_func(rgb[0], rgb[1], rgb[2]);

                rgb[0] = output_rgb[0];
                rgb[1] = output_rgb[1];
                rgb[2] = output_rgb[2];
            }
        }
    });
    save_tile(&tile_data, (width, height));
    let duration = start.elapsed();
    println!("took {:?}", duration);
}
