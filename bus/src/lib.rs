#[allow(dead_code)]
mod error;
pub use error::*;

pub mod address;
pub mod packet;
pub mod router;
pub mod switch;
pub mod wire;
pub mod domain;

mod types;
pub use types::*;

#[cfg(test)]
mod mod_test;
