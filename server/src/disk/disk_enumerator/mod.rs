#[cfg(test)]
mod mod_test;

pub mod disk_enumerator;
pub use disk_enumerator::{Builder, DiskEnumerator};

pub mod event;
pub use event::Event;

use crate::core::bus;
use crate::core::EventEnum;
use crate::shutdown::ShutdownReceiver;

pub struct DiskEnumeratorPoller {
    disk_enumerator: DiskEnumerator,
    bus: bus::Bus<EventEnum>,
}

impl DiskEnumeratorPoller {
    pub fn new(disk_enumerator: DiskEnumerator, bus: bus::Bus<EventEnum>) -> Self {
        Self {
            disk_enumerator,
            bus,
        }
    }

    pub fn spawn(self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        let mut bus_listener = self.bus.receiver();
        let bus_sender = self.bus.sender();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = bus_listener.recv() => {
                        let reply_event = self.disk_enumerator.event_process(event).unwrap();
                        if let Some(event) = reply_event {
                            let _ = bus_sender.send(event);
                        }
                    }
                    _ = shutdown.wait() => {
                        break;
                    }
                }
            }
        })
    }
}
