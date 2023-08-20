use gdal::spatial_ref::CoordTransform;
use crate::{geojson::PolygonGeometry, utils::Error, utils::ScriptError, utils::Result};

#[derive(Clone)]
pub struct BoundingBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl BoundingBox {
    pub fn transform(&self, transform: &CoordTransform) -> Result<BoundingBox> {
        let mut bounds = [self.xmin, self.ymin, self.xmax, self.ymax];
        bounds = transform.transform_bounds(&bounds, 21)?;
        Ok(BoundingBox {
            xmin: bounds[0],
            ymin: bounds[1],
            xmax: bounds[2],
            ymax: bounds[3],
        })
    }

    pub fn to_array(&self) -> [f64; 4] {
        [self.xmin, self.ymin, self.xmax, self.ymax]
    }

    /// Extend self to contain other
    pub fn extend(&mut self, other: &BoundingBox) {
        self.xmin = self.xmin.min(other.xmin);
        self.xmax = self.xmax.max(other.xmax);
        self.ymin = self.ymin.min(other.ymin);
        self.ymax = self.ymax.max(other.ymax);
    }

    /// Return the union of all the passed bounding boxes. Returns None if 0 bounding boxes are passed
    // TODO: Return ScriptError
    pub fn union(boxes: &[BoundingBox]) -> Result<BoundingBox> {
        if boxes.is_empty() {
            Err(Error::ScriptError(ScriptError::NotEnoughinputs))
        } else {
            let mut bbox = boxes[0].clone();
            for other in boxes.iter().skip(1) {
                bbox.extend(other);
            }
            Ok(bbox)
        }
    }
}

impl From<BoundingBox> for PolygonGeometry {
    fn from(bbox: BoundingBox) -> Self {
        PolygonGeometry::from_exterior(vec![
            [bbox.xmin, bbox.ymin],
            [bbox.xmax, bbox.ymin],
            [bbox.xmax, bbox.ymax],
            [bbox.xmin, bbox.ymax],
            [bbox.xmin, bbox.ymin],
        ])
    }
}
