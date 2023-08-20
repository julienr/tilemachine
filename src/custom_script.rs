use crate::bbox::BoundingBox;
use crate::geojson::PolygonGeometry;
use crate::utils::ImageData;
use crate::utils::Result;
use crate::xyz::{extract_tile, TileCoords, TILE_SIZE};
use gdal::Dataset;
use crate::raster;
use serde::Deserialize;
use std::collections::HashMap;
use std::{sync::atomic::AtomicBool, sync::atomic::Ordering};

#[derive(Deserialize)]
pub struct CustomScript {
    script: String,
    inputs: HashMap<String, String>,
}

impl CustomScript {
    pub fn new_from_str(json_str: &str) -> Result<CustomScript> {
        let s: CustomScript = serde_json::from_str(json_str)?;
        Ok(s)
    }

    pub fn execute_on_tile(
        &self,
        coords: &TileCoords,
        open_dataset_fn: &dyn Fn(&str) -> Result<Dataset>,
    ) -> Result<ImageData<u8>> {
        let mut engine = JSEngine::default();
        let mut coll = ImageDataCollection::<f64>::new(TILE_SIZE as usize);
        for (name, filename) in self.inputs.iter() {
            let ds = open_dataset_fn(filename)?;
            let image_data = extract_tile(&ds, coords);
            // Convert from u8 to f64 for computations
            let data_f64 = image_data.data.iter().map(|e| *e as f64).collect();
            let image_data = ImageData::from_vec(
                image_data.width,
                image_data.height,
                image_data.channels,
                data_f64,
            );
            coll.images.push((name.to_string(), image_data));
        }
        let output = engine.execute_on_tile(&self.script, &coll);
        Ok(output)
    }


    pub fn get_bounds(
        &self,
        open_dataset_fn: &dyn Fn(&str) -> Result<Dataset>
    ) -> Result<BoundingBox> {
        let mut bboxes: Vec<BoundingBox> = vec![];
        for (_name, filename) in self.inputs.iter() {
            let ds = open_dataset_fn(filename)?;
            bboxes.push(raster::wgs84_bbox(&ds)?);
        }

        BoundingBox::union(&bboxes)
    }

    pub fn get_bounds_as_polygon(
        &self,
        open_dataset_fn: &dyn Fn(&str) -> Result<Dataset>
    ) -> Result<PolygonGeometry> {
        Ok(self.get_bounds(open_dataset_fn)?.into())
    }

}

static mut PLATFORM_INITIALIZED: AtomicBool = AtomicBool::new(false);

struct JSEngine {
    isolate: v8::OwnedIsolate,
}

impl Default for JSEngine {
    fn default() -> Self {
        // Doing platform initialization twice seems to lead to "Invalid global state"
        // so it looks like we need a singleton to ensure this is done exactly once
        let needs_initialization = unsafe {
            PLATFORM_INITIALIZED.compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        };
        if needs_initialization.is_ok() {
            println!("initializing v8");
            let platform = v8::new_default_platform(0, false).make_shared();
            v8::V8::initialize_platform(platform);
            v8::V8::initialize();
        }

        let isolate = v8::Isolate::new(Default::default());
        JSEngine { isolate }
    }
}

fn run_on_pixel(
    func: &v8::Local<v8::Function>,
    args: Vec<(String, &[f64])>,
    context_scope: &mut v8::ContextScope<v8::HandleScope>,
) -> [u8; 4] {
    let call_scope = &mut v8::HandleScope::new(context_scope);
    let args: Vec<v8::Local<'_, v8::Value>> = args
        .iter()
        .map(|(_name, pixel_values)| {
            let elements: Vec<v8::Local<'_, v8::Value>> = pixel_values
                .iter()
                .map(|v| v8::Number::new(call_scope, *v).into())
                .collect();
            v8::Array::new_with_elements(call_scope, &elements[..]).into()
        })
        .collect();
    let function_this: v8::Local<'_, v8::Value> = v8::null(call_scope).into();
    let return_value = func.call(call_scope, function_this, &args).unwrap();
    let return_scope = &mut v8::HandleScope::new(call_scope);
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
    [
        extract_channel(0),
        extract_channel(1),
        extract_channel(2),
        extract_channel(3),
    ]
}

impl JSEngine {
    fn compile_function<F>(&mut self, code: &str, args_names: Vec<&String>, callback: &mut F)
    where
        F: FnMut(&v8::Local<v8::Function>, &mut v8::ContextScope<v8::HandleScope>),
    {
        let handle_scope = &mut v8::HandleScope::new(&mut self.isolate);
        let context = v8::Context::new(handle_scope);
        let context_scope = &mut v8::ContextScope::new(handle_scope, context);
        let code = v8::String::new(context_scope, code).unwrap();

        let arg_names: Vec<v8::Local<'_, v8::String>> = args_names
            .iter()
            .map(|name| v8::String::new(context_scope, name).unwrap())
            .collect();

        let function = v8::script_compiler::compile_function(
            context_scope,
            v8::script_compiler::Source::new(code, None),
            &arg_names,
            &[],
            v8::script_compiler::CompileOptions::NoCompileOptions,
            v8::script_compiler::NoCacheReason::NoReason,
        )
        .unwrap();
        callback(&function, context_scope);
    }

    pub fn execute_on_tile(
        &mut self,
        code: &str,
        inputs: &ImageDataCollection<f64>,
    ) -> ImageData<u8> {
        let arg_names: Vec<&String> = inputs.images.iter().map(|(name, _data)| name).collect();
        let mut output = ImageData::<u8>::new(inputs.tile_size, inputs.tile_size, 4);
        self.compile_function(code, arg_names, &mut |function, context_scope| {
            // let call_scope = &mut v8::HandleScope::new(context_scope);
            for i in 0..output.height {
                for j in 0..output.width {
                    let mut args: Vec<(String, &[f64])> = vec![];
                    for (name, image) in inputs.images.iter() {
                        let start_index = i * image.width * image.channels + j * image.channels;
                        let end_index = start_index + image.channels;
                        let val = &image.data[start_index..end_index];
                        args.push((name.clone(), val));
                    }
                    let out_val = run_on_pixel(function, args, context_scope);
                    let out_start_index = i * output.width * output.channels + j * output.channels;
                    output.data[out_start_index] = out_val[0];
                    output.data[out_start_index + 1] = out_val[1];
                    output.data[out_start_index + 2] = out_val[2];
                    output.data[out_start_index + 3] = out_val[3];
                }
            }
        });
        output
    }
}

pub struct ImageDataCollection<T> {
    // We use a vector and not a hashmap here to guarantee ordering
    pub images: Vec<(String, ImageData<T>)>,
    pub tile_size: usize,
}

impl<T> ImageDataCollection<T> {
    pub fn new(tile_size: usize) -> ImageDataCollection<T> {
        ImageDataCollection {
            images: vec![],
            tile_size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_on_tile_1() {
        let code = "return [3 * rgb[1], rgb[0], dsm[0]]";
        let mut engine = JSEngine::default();
        let mut coll = ImageDataCollection::<f64>::new(2);
        coll.images.push((
            "rgb".to_owned(),
            ImageData::<f64>::from_vec(2, 2, 2, vec![0.0, 5.0, 4.0, 1.0, 3.0, 2.0, 7.0, 8.0]),
        ));
        coll.images.push((
            "dsm".to_owned(),
            ImageData::<f64>::from_vec(2, 2, 1, vec![42.0, 43.0, 44.0, 45.0]),
        ));

        let out_image = engine.execute_on_tile(code, &coll);
        assert!(out_image.width == 2);
        assert!(out_image.height == 2);
        assert!(out_image.pixel_data(0, 0)[0] == 15);
        assert!(out_image.pixel_data(0, 0)[1] == 0);
        assert!(out_image.pixel_data(0, 0)[2] == 42);
        assert!(out_image.pixel_data(1, 1)[0] == 24);
        assert!(out_image.pixel_data(1, 1)[1] == 7);
        assert!(out_image.pixel_data(1, 1)[2] == 45);
    }
}
