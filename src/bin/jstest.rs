// Example of using v8 to do custom pixel-by-pixel processing (as a JS function)
// on a tile
// Usage:
// `cargo run --bin jstest && eog out.png`
use std::fs::File;
use std::io::prelude::*;
use std::io::BufWriter;
use std::time::Instant;

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
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(Default::default());
    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let (mut tile_data, width, height) = load_tile();
    let code = v8::String::new(scope, "return [r * 2, g, b]").unwrap();
    let args = [
        v8::String::new(scope, "r").unwrap(),
        v8::String::new(scope, "g").unwrap(),
        v8::String::new(scope, "b").unwrap(),
    ];
    let function = v8::script_compiler::compile_function(
        scope,
        v8::script_compiler::Source::new(code, None),
        &args,
        &[],
        v8::script_compiler::CompileOptions::NoCompileOptions,
        v8::script_compiler::NoCacheReason::NoReason,
    )
    .unwrap();
    let start = Instant::now();
    for i in 0..height {
        for j in 0..width {
            let function_scope = &mut v8::HandleScope::new(scope);
            let rgb = &mut tile_data[i * width * 3 + j * 3..i * width * 3 + (j + 1) * 3];
            let args = [
                v8::Number::new(function_scope, rgb[0] as f64).into(),
                v8::Number::new(function_scope, rgb[1] as f64).into(),
                v8::Number::new(function_scope, rgb[2] as f64).into(),
            ];
            let function_this: v8::Local<'_, v8::Value> = v8::null(function_scope).into();
            let return_value = function.call(function_scope, function_this, &args).unwrap();
            let return_scope = &mut v8::HandleScope::new(function_scope);
            if !return_value.is_array() {
                panic!(
                    "Expected an array as return type, got {:?}",
                    return_value.type_repr()
                );
            }
            let return_array = v8::Local::<v8::Array>::try_from(return_value).unwrap();

            let mut extract_channel = |i: u32| -> u8 {
                let mut v = return_array
                    .get_index(return_scope, i)
                    .unwrap()
                    .number_value(return_scope)
                    .unwrap();
                if v > 255.0 {
                    v = 255.0;
                }
                v as u8
            };

            rgb[0] = extract_channel(0);
            rgb[1] = extract_channel(1);
            rgb[2] = extract_channel(2);
        }
    }
    save_tile(&tile_data, (width, height));
    let duration = start.elapsed();
    println!("took {:?}", duration);
}
