use crate::bbox::BoundingBox;
use crate::raster::wgs84_bbox;
use crate::source::Source;
use crate::utils::Error::GdalError;
use crate::utils::Result;
use gdal::Dataset;

pub struct GdalSource {
    ds: Dataset,
}

impl GdalSource {
    pub fn from_file(filename: &str) -> Result<GdalSource> {
        let ds = Dataset::open(filename)?;
        Ok(GdalSource { ds })
    }

    pub fn from_blobstore(blobname: &str) -> Result<GdalSource> {
        let mut vsi_path = "/vsis3/".to_owned();
        vsi_path.push_str(blobname);
        let ds = Dataset::open(vsi_path.as_str())?;
        Ok(GdalSource { ds })
    }
}

impl Source for GdalSource {
    fn num_bands(&self) -> usize {
        self.ds.raster_count() as usize
    }

    fn reproject_to(&self, target_ds: &Dataset) -> Result<()> {
        match gdal::raster::reproject(&self.ds, target_ds) {
            Ok(_) => Ok(()),
            Err(e) => Err(GdalError(e)),
        }
    }

    fn wgs84_bbox(&self) -> Result<BoundingBox> {
        wgs84_bbox(&self.ds)
    }
}
