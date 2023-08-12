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
