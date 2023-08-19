use crate::utils::ImageData;
use gdal::raster::ResampleAlg;
use gdal::Dataset;
use std::io::BufWriter;

// TODO: This require 'rasterIO' to be exposed on the Dataset, see
// https://github.com/georust/gdal/pull/374
pub fn read_ds_at_once(ds: &Dataset) -> ImageData<u8> {
    let nbands = ds.raster_count() as usize;
    let size = ds.raster_size();
    let buf = ds
        .read_as::<u8>(
            (0, 0),
            (size.0, size.1),
            (size.0, size.1),
            Some(ResampleAlg::Bilinear),
            gdal::ImageInterleaving::Pixel,
            gdal::BandSelection::All,
        )
        .unwrap()
        .data;
    ImageData::from_vec(size.0, size.1, nbands, buf)
}

// Reads the whole dataset into a buffer
// Returns the buffer and its size (width, height)
#[allow(dead_code)]
pub fn read_ds_band_by_band(ds: &Dataset) -> ImageData<u8> {
    let nbands = ds.raster_count() as usize;
    let size = ds.raster_size();
    let mut image_data = ImageData::<u8>::new(size.0, size.1, nbands);
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
            image_data.data[j * nbands + (i - 1)] = *e;
        }
    }
    image_data
}

// Reads the whole dataset into a PNG image
pub fn image_bytes_to_png(image_data: &ImageData<u8>) -> Vec<u8> {
    let mut out_buf = Vec::new();
    {
        let w = BufWriter::new(&mut out_buf);
        let mut encoder = png::Encoder::new(w, image_data.width as u32, image_data.height as u32);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(&image_data.data).unwrap();
    }
    out_buf
}
