/// Custom error type for tilemachine
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    GdalError(gdal::errors::GdalError),
    SerdeError(serde_json::Error),
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::SerdeError(value)
    }
}

impl From<gdal::errors::GdalError> for Error {
    fn from(value: gdal::errors::GdalError) -> Self {
        Error::GdalError(value)
    }
}

pub struct ImageData<T> {
    pub width: usize,
    pub height: usize,
    pub channels: usize,
    // Data stored in row-major order, channels last
    pub data: Vec<T>,
}

impl<T: Default + Clone> ImageData<T> {
    pub fn new(width: usize, height: usize, channels: usize) -> ImageData<T> {
        ImageData {
            width,
            height,
            channels,
            data: vec![T::default(); width * height * channels],
        }
    }

    pub fn from_vec(width: usize, height: usize, channels: usize, data: Vec<T>) -> ImageData<T> {
        assert!(data.len() == width * height * channels);
        ImageData {
            width,
            height,
            channels,
            data,
        }
    }

    pub fn pixel_data(&self, i: usize, j: usize) -> &[T] {
        let start_index = i * self.width * self.channels + j * self.channels;
        let end_index = start_index + self.channels;
        &self.data[start_index..end_index]
    }
}
