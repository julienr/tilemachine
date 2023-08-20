use crate::bbox::BoundingBox;
use crate::utils::Result;
use gdal::{spatial_ref::CoordTransform, spatial_ref::SpatialRef, Dataset};
use gdal_sys::OSRAxisMappingStrategy;

fn raster_local_bbox(ds: &Dataset) -> Result<BoundingBox> {
    let geot = ds.geo_transform()?;
    let (width, height) = ds.raster_size();

    Ok(BoundingBox {
        xmin: geot[0],
        ymin: geot[3],
        xmax: geot[0] + width as f64 * geot[1],
        ymax: geot[3] + height as f64 * geot[5],
    })
}

pub fn raster_projected_bbox(ds: &Dataset, epsg: u32) -> Result<BoundingBox> {
    let local_bbox = raster_local_bbox(ds)?;
    let raster_srs = ds.spatial_ref()?;
    raster_srs.set_axis_mapping_strategy(OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    let target_srs = SpatialRef::from_epsg(epsg)?;
    target_srs.set_axis_mapping_strategy(OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    let transform = CoordTransform::new(&raster_srs, &target_srs)?;
    local_bbox.transform(&transform)
}

pub fn wgs84_bbox(ds: &Dataset) -> Result<BoundingBox> {
    raster_projected_bbox(ds, 4326)
}
