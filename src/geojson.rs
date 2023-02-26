use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct PolygonGeometry {
    #[serde(rename(serialize = "type"))]
    pub geom_type: String,
    pub coordinates: Vec<Vec<[f64; 2]>>,
}

impl PolygonGeometry {
    pub fn from_exterior(coords: Vec<[f64; 2]>) -> Self {
        Self {
            geom_type: "polygon".to_owned(),
            coordinates: vec![coords],
        }
    }
}
