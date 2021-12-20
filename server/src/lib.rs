pub mod core;
pub mod result;
pub mod shutdown;

pub mod event_generator;

#[cfg(target_os = "linux")]
pub mod disk;

#[cfg(target_os = "linux")]
pub mod udev;
