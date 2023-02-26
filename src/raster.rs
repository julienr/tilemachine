use crate::bbox::BoundingBox;
use crate::geojson::PolygonGeometry;
use gdal::{errors::GdalError, spatial_ref::CoordTransform, spatial_ref::SpatialRef, Dataset};
use gdal_sys::OSRAxisMappingStrategy;

fn raster_local_bbox(ds: &Dataset) -> Result<BoundingBox, GdalError> {
    let geot = ds.geo_transform()?;
    let (width, height) = ds.raster_size();

    Ok(BoundingBox {
        xmin: geot[0],
        ymin: geot[3],
        xmax: geot[0] + width as f64 * geot[1],
        ymax: geot[3] + height as f64 * geot[5],
    })
}

fn raster_projected_bbox(ds: &Dataset, epsg: u32) -> Result<BoundingBox, GdalError> {
    let local_bbox = raster_local_bbox(ds)?;
    let raster_srs = ds.spatial_ref()?;
    raster_srs.set_axis_mapping_strategy(OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    let target_srs = SpatialRef::from_epsg(epsg)?;
    target_srs.set_axis_mapping_strategy(OSRAxisMappingStrategy::OAMS_TRADITIONAL_GIS_ORDER);
    let transform = CoordTransform::new(&raster_srs, &target_srs)?;
    local_bbox.transform(&transform)
}

pub fn bounds(ds: &Dataset) -> Result<PolygonGeometry, GdalError> {
    let bbox = raster_projected_bbox(ds, 4326)?;
    Ok(PolygonGeometry::from_exterior(vec![
        [bbox.xmin, bbox.ymin],
        [bbox.xmax, bbox.ymin],
        [bbox.xmax, bbox.ymax],
        [bbox.xmin, bbox.ymax],
        [bbox.xmin, bbox.ymin],
    ]))
}
