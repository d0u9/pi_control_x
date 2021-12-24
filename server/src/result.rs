use std::convert::From;
use std::error;
use std::fmt;
use std::fmt::Debug;
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

use std::io;

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error {
            msg: format!("[IO]: {:?}", e),
        }
    }
}

use std::net;
impl From<net::AddrParseError> for Error {
    fn from(e: net::AddrParseError) -> Self {
        Error {
            msg: format!("[AddrParseError]: {:?}", e),
        }
    }
}

#[cfg(target_os = "linux")]
use lfs_core;

#[cfg(target_os = "linux")]
impl From<lfs_core::Error> for Error {
    fn from(e: lfs_core::Error) -> Self {
        Error {
            msg: format!("[LFS_CORE]: {:?}", e),
        }
    }
}

