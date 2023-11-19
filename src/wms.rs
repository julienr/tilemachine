use crate::bbox::BoundingBox;
use crate::custom_script::CustomScript;
use crate::source::Source;
use crate::utils::Result;
use handlebars::Handlebars;
use serde_json::json;

pub fn capabilities(
    script: &CustomScript,
    open_source_fn: &dyn Fn(&str) -> Result<Box<dyn Source>>,
) -> Result<String> {
    let bbox = script.get_bounds(open_source_fn)?;
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
