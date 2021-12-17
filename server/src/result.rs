use lfs_core;
use std::convert::From;
use std::error;
use std::fmt;
use std::fmt::Debug;
use std::io;
use std::result;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    msg: String,
}

impl Error {
    pub fn with_str(s: &str) -> Self {
        Self { msg: s.to_owned() }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error {
            msg: format!("[IO]: {:?}", e),
        }
    }
}

impl From<lfs_core::Error> for Error {
    fn from(e: lfs_core::Error) -> Self {
        Error {
            msg: format!("[LFS_CORE]: {:?}", e),
        }
    }
}
