use crate::jsengine::{ImageData, ImageDataCollection};
use crate::utils::Result;
use crate::{
    jsengine::JSEngine,
    xyz::{extract_tile, TileCoords, TILE_SIZE},
};
use gdal::Dataset;
use serde::Deserialize;
use std::collections::HashMap;

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
        engine: &mut JSEngine,
    ) -> Result<ImageData<u8>> {
        let mut coll = ImageDataCollection::<f64>::new(TILE_SIZE as usize);
        for (name, filename) in self.inputs.iter() {
            let ds = open_dataset_fn(filename)?;
            let (data_u8, size) = extract_tile(&ds, coords);
            let data_f64 = data_u8.iter().map(|e| *e as f64).collect();
            // TODO: Move ImageData to xyz
            let image_data = ImageData::from_vec(
                size.0, size.1, 4, // RGBA
                data_f64,
            );
            coll.images.push((name.to_string(), image_data));
        }
        let output = engine.execute_on_tile(&self.script, &coll);
        Ok(output)
    }
}
