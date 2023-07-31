use crate::bbox::BoundingBox;
use crate::raster::raster_projected_bbox;
use gdal::Dataset;
use handlebars::Handlebars;
use serde_json::json;
use std::error::Error;

pub fn capabilities(image_name: &str, ds: &Dataset) -> Result<String, Box<dyn Error>> {
    let bbox = raster_projected_bbox(ds, 4326)?;
    get_capabilities_xml(image_name, bbox)
}

fn get_capabilities_xml(
    layer_name: &str,
    layer_bbox: BoundingBox,
) -> Result<String, Box<dyn Error>> {
    let reg = Handlebars::new();
    let tpl = include_str!("wms_capabilities.xml");
    reg.render_template(
        tpl,
        &json!({"service_name": "tilemachine", "layer_name": layer_name, "bbox": layer_bbox.to_array()}),
    )
    .map_err(|e| e.into())
}
