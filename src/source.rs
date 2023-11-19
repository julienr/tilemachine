/// A source is an abstraction over a raster datasource. It can be many things:
/// - A gdal Dataset opened from a raster file (local or from blobstore with GDAL VSI infrastructure)
/// - An upstream WMS server
/// - An upstream XYZ server
mod gdal_source;
use crate::bbox::BoundingBox;
use crate::utils::{Error, Result};
use gdal::Dataset;
use gdal_source::GdalSource;

pub trait Source {
    fn num_bands(&self) -> usize;
    fn reproject_to(&self, target_ds: &Dataset) -> Result<()>;

    fn wgs84_bbox(&self) -> Result<BoundingBox>;
}

pub fn open_source(path: &str) -> Result<Box<dyn Source>> {
    match path.split_once(':') {
        Some(("file", filename)) => {
            let source = GdalSource::from_file(filename)?;
            Ok(Box::new(source))
        }
        Some(("s3", s3_path)) => {
            let source = GdalSource::from_blobstore(s3_path)?;
            Ok(Box::new(source))
        }
        Some(("wms", wms_path)) => {
            println!("WMS!! {}", wms_path);
            Result::Err(Error::InvalidPath(format!("WMS path: {}", wms_path)))
        }
        _ => {
            println!("Invalid path {}", path);
            Result::Err(Error::InvalidPath(path.to_string()))
        }
    }
}
