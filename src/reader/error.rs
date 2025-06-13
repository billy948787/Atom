use crate::reader;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FileError {
    #[error("File not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Invalid file format: {0}")]
    InvalidFormat(String),

    #[error("Unexpected end of file")]
    UnexpectedEndOfFile,

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Format error: {0} of {1} in file {2}")]
    FormatError(String, reader::file_type::FileType, usize),
}
