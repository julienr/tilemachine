use gdal::{errors::GdalError, spatial_ref::CoordTransform};

pub struct BoundingBox {
    pub xmin: f64,
    pub ymin: f64,
    pub xmax: f64,
    pub ymax: f64,
}

impl BoundingBox {
    pub fn transform(self, transform: &CoordTransform) -> Result<BoundingBox, GdalError> {
        let mut bounds = [self.xmin, self.ymin, self.xmax, self.ymax];
        bounds = transform.transform_bounds(&bounds, 21)?;
        Ok(BoundingBox {
            xmin: bounds[0],
            ymin: bounds[1],
            xmax: bounds[2],
            ymax: bounds[3],
        })
    }
}
