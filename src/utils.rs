use handlebars::RenderError;
use std::io::BufWriter;

/// Custom error type for tilemachine
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum ScriptError {
    NotEnoughinputs,
    CompilationError(String),
    RuntimeError(String),
    InvalidReturnType,
}

#[derive(Debug)]
pub enum Error {
    GdalError(gdal::errors::GdalError),
    SerdeError(serde_json::Error),
    ScriptError(ScriptError),
    HandlebarsError(RenderError),
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

impl From<RenderError> for Error {
    fn from(value: RenderError) -> Self {
        Error::HandlebarsError(value)
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

impl ImageData<u8> {
    /// Encode this image data as PNG and return the bytes
    pub fn to_png(&self) -> Vec<u8> {
        let mut out_buf = Vec::new();
        {
            let w = BufWriter::new(&mut out_buf);
            let mut encoder = png::Encoder::new(w, self.width as u32, self.height as u32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&self.data).unwrap();
        }
        out_buf
    }
}
