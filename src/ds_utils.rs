use gdal::raster::ResampleAlg;
use gdal::Dataset;
use std::io::BufWriter;

/*
// TODO: This require 'rasterIO' to be exposed on the Dataset, see
// https://github.com/georust/gdal/pull/374
fn read_ds_at_once(ds: &Dataset) -> Vec<u8> {
    ds.read_as::<u8>(
        (0, 0),
        (TILE_SIZE as usize, TILE_SIZE as usize),
        (TILE_SIZE as usize, TILE_SIZE as usize),
        Some(ResampleAlg::Bilinear),
    )
    .unwrap()
    .data
}
*/

// Reads the whole dataset into a buffer
// Returns the buffer and its size (width, height)
pub fn read_ds_band_by_band(ds: &Dataset) -> (Vec<u8>, (usize, usize)) {
    let nbands = ds.raster_count() as usize;
    let size = ds.raster_size();
    let mut buf: Vec<u8> = vec![0; size.0 * size.1 * nbands];
    for i in 1..nbands + 1 {
        let band = ds.rasterband(i as isize).unwrap();
        let data = band
            .read_as::<u8>(
                (0, 0),
                (size.0, size.1),
                (size.0, size.1),
                Some(ResampleAlg::Bilinear),
            )
            .unwrap()
            .data;
        // Place pixels in buf
        for (j, e) in data.iter().enumerate() {
            buf[j * nbands + (i - 1)] = *e;
        }
    }
    (buf, size)
}

// Reads the whole dataset into a PNG image
pub fn read_ds_into_png(ds: &Dataset) -> Vec<u8> {
    let (buf, size) = read_ds_band_by_band(ds);
    let mut out_buf = Vec::new();
    {
        let w = BufWriter::new(&mut out_buf);
        let mut encoder = png::Encoder::new(w, size.0 as u32, size.1 as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&buf).unwrap();
    }
    out_buf
}
