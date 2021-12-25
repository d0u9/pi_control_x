use ::std::collections::HashMap;
use ::tokio::sync::broadcast;

use super::bus;

#[cfg(target_os = "linux")]
use crate::disk::disk_enumerator;
#[cfg(target_os = "linux")]
use crate::disk::mounter;
#[cfg(target_os = "linux")]
use crate::udev;

use crate::disk::snapshot;
use crate::grpc;

#[derive(Clone, Debug)]
pub enum EventEnum {
    NULL,
    #[cfg(target_os = "linux")]
    Udev(udev::Event),
    #[cfg(target_os = "linux")]
    Mounter(mounter::Event),
    #[cfg(target_os = "linux")]
    DiskEnumerator(disk_enumerator::Event),

    Snapshot(snapshot::Event),
    Grpc(grpc::Event),
}

pub struct Core {
    bus: bus::Bus<EventEnum>,
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
