use std::error;
use std::fmt;
use std::result;
use std::convert::From;
use std::io;

pub type Result<T> = result::Result<T, Error>;

#[derive(Debug)]
pub struct Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl error::Error for Error {}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error{}
    }
}
