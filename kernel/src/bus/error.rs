use std::convert::From;

pub type Result<T> = std::result::Result<T, Error>;

pub struct Error {
    msg: String,
}

