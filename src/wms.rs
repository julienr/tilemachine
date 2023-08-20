use crate::bbox::BoundingBox;
use crate::custom_script::CustomScript;
use crate::utils::Result;
use gdal::Dataset;
use handlebars::Handlebars;
use serde_json::json;

pub fn capabilities(
    script: &CustomScript,
    open_dataset_fn: &dyn Fn(&str) -> Result<Dataset>,
) -> Result<String> {
    let bbox = script.get_bounds(open_dataset_fn)?;
    // TODO: Give specific/unique names
    get_capabilities_xml("image", bbox)
}

fn get_capabilities_xml(layer_name: &str, layer_bbox: BoundingBox) -> Result<String> {
    let reg = Handlebars::new();
    let tpl = include_str!("wms_capabilities.xml");
    reg.render_template(
        tpl,
        &json!({"service_name": "tilemachine", "layer_name": layer_name, "bbox": layer_bbox.to_array()}),
    )
    .map_err(|e| e.into())
}
