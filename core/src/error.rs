use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ZhangError {
    #[error("date is invalid")]
    InvalidDate,
    #[error("account is invalid")]
    InvalidAccount,
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("file error: {e}")]
    FileError { e: std::io::Error, path: PathBuf },

    #[error("pest error: {0}")]
    PestError(String),
    #[error("cannot found option given key: {0}")]
    OptionNotFound(String),
}

pub trait IoErrorIntoZhangError<T> {
    fn with_path(self, path: &Path) -> Result<T, ZhangError>;
}

impl<T> IoErrorIntoZhangError<T> for Result<T, std::io::Error> {
    fn with_path(self, path: &Path) -> Result<T, ZhangError> {
        self.map_err(|e| ZhangError::FileError { e, path: path.to_path_buf() })
    }
}
