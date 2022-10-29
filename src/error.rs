#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    IoError(std::io::Error),
    InvalidSave,
    IncompatibleVersion(u8),
}

impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Self::IoError(error)
    }
}
