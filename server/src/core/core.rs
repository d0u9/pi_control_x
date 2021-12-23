use ::std::collections::HashMap;
use ::tokio::sync::broadcast;

use super::bus;

#[cfg(target_os = "linux")]
use crate::udev;
#[cfg(target_os = "linux")]
use crate::disk::mounter;

use crate::disk::disk_enumerator;

#[derive(Clone, Debug)]
pub enum EventEnum {
    NULL,
    #[cfg(target_os = "linux")]
    Udev(udev::Event),
    #[cfg(target_os = "linux")]
    Mounter(mounter::Event),
    DiskEnumerator(disk_enumerator::Event),
}

pub struct Core {
    bus: bus::Bus,
}

impl Core {
    pub fn new() -> Self {
        Core {
            bus: bus::Bus::new(),
        }
    }

    pub fn enable_source(mut self, source: &str) -> Self {
        self
    }
}
