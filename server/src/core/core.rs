use ::std::collections::HashMap;
use ::tokio::sync::broadcast;

use super::bus;
use crate::udev;

#[derive(Clone, Debug)]
pub enum EventEnum {
    NULL,
    Udev(udev::Event),
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
