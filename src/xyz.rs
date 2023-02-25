use png;
use std::f64::consts::PI;
use std::io::BufWriter;

use gdal::raster::ResampleAlg;
use gdal::{spatial_ref::SpatialRef, Dataset, DriverManager};

// This is the WGS_1984 spheroid radius in meters
// https://epsg.io/3857
const EARTH_RADIUS: f64 = 6378137.0;
const EQUATOR_LENGTH_M: f64 = EARTH_RADIUS * 2.0 * PI;

// Note that here 256 is *NOT* the tile image size, but the number of pixels covered at zoom level
// 0, which is 256 by definition of the XYZ system, even if we would return highdpi tiles of
// 512x512
const INITIAL_RESOLUTION: f64 = EQUATOR_LENGTH_M / 256.0;

// This maps [0, EQUATOR_LENGTH_M] to [-EQUATOR_LENGTH_M / 2.0, EQUATOR_LENGTH_M / 2.0], which
// corresponds to the projected bounds [-20037508.34 20037508.34]
// Note that https://epsg.io/3857 reports  20048966.1 as the bound for Y, but this is because
// it is technically only valid up to 85.06 degree and here we compute up to 90 degrees
const EPSG_3857_ORIGIN_SHIFT: f64 = -EQUATOR_LENGTH_M / 2.0;

const TILE_SIZE: u64 = 256;

// Returns the resolution in meters at the equator for the given zoom level. Note that EPSG:3857
// has quite large resolution deformation as you move away from the equator
// https://wiki.openstreetmap.org/wiki/Slippy_map_tilenames#Resolution_and_Scale
fn resolution_at_zoom(zoom: u64) -> f64 {
    INITIAL_RESOLUTION / (2.0_f64.powf(zoom as f64))
}

// Converts from xy coordinates in given zoom level of the pyramid to EPSG:3857
fn xyz_to_3857_meters(zx: u64, zy: u64, zoom: u64) -> (f64, f64) {
    let res = resolution_at_zoom(zoom);
    (
        zx as f64 * res + EPSG_3857_ORIGIN_SHIFT,
        zy as f64 * res + EPSG_3857_ORIGIN_SHIFT,
    )
}

struct TileBounds {
    pub xmin: f64,
    #[allow(dead_code)]
    pub ymin: f64,
    #[allow(dead_code)]
    pub xmax: f64,
    pub ymax: f64,
}

fn compute_tile_bounds(x: u64, y: u64, zoom: u64) -> TileBounds {
    let (xmin, ymin) = xyz_to_3857_meters(x * TILE_SIZE, y * TILE_SIZE, zoom);
    let (xmax, ymax) = xyz_to_3857_meters((x + 1) * TILE_SIZE, (y + 1) * TILE_SIZE, zoom);
    TileBounds {
        xmin,
        ymin,
        xmax,
        ymax,
    }
}

pub fn extract_tile(ds: &Dataset, x: u64, y: u64, zoom: u64) -> Vec<u8> {
    // TODO: Early return if tile out of raster
    // TODO: Early return if raster invisible in tile (covers too little)
    let tile_srs = SpatialRef::from_epsg(3857).unwrap();
    let drv = DriverManager::get_driver_by_name("MEM").unwrap();
    let mut tile_ds = drv
        .create("", TILE_SIZE as isize, TILE_SIZE as isize, 4)
        .unwrap();
    tile_ds
        .rasterband(4)
        .unwrap()
        .set_color_interpretation(gdal::raster::ColorInterpretation::AlphaBand)
        .unwrap();

    let tile_bounds = compute_tile_bounds(x, y, zoom);

    let pixel_size = resolution_at_zoom(zoom);
    let tile_geo = [
        tile_bounds.xmin,
        pixel_size,
        0.0,
        tile_bounds.ymax,
        0.0,
        -pixel_size,
    ];

    tile_ds.set_geo_transform(&tile_geo).unwrap();
    tile_ds.set_spatial_ref(&tile_srs).unwrap();

    // TODO: Warp from ds into tile_ds, read as RGBA and encode as PNG
    gdal::raster::reproject(ds, &tile_ds).unwrap();
    println!("ds={:?}, tile_ds={:?}", ds, tile_ds);
    println!("extracting_tile for x={:?}, y={:?}, zoom={:?}", x, y, zoom);
    let buf = tile_ds
        .read_as::<u8>(
            (0, 0),
            (TILE_SIZE as usize, TILE_SIZE as usize),
            (TILE_SIZE as usize, TILE_SIZE as usize),
            Some(ResampleAlg::Bilinear),
        )
        .unwrap();
    println!("buf len={:?}", buf.data.len());
    {
        let mut out_buf = Vec::new();
        {
            let w = BufWriter::new(&mut out_buf);
            let mut encoder = png::Encoder::new(w, TILE_SIZE as u32, TILE_SIZE as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&buf.data).unwrap();
        }
        return out_buf;
    }
}
