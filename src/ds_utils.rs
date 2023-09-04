use crate::utils::ImageData;
use gdal::raster::ResampleAlg;
use gdal::Dataset;

// TODO: This require 'rasterIO' to be exposed on the Dataset, see
// https://github.com/georust/gdal/pull/374
pub fn read_ds_at_once(ds: &Dataset) -> ImageData<f64> {
    let nbands = ds.raster_count() as usize;
    let size = ds.raster_size();
    let buf = ds
        .read_as::<f64>(
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
