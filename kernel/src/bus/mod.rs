#![cfg(test)]
mod error;
pub use error::*;

mod address;
mod packet;
mod router;
mod switch;
mod wire;

#[cfg(test)]
mod mod_test;
