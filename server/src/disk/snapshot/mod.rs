#[cfg(test)]
mod mod_test;

mod snapshot;
pub use snapshot::{Builder, Snapshot};

mod event;
pub use event::Event;

use crate::core::bus;
use crate::shutdown::ShutdownReceiver;

pub struct SnapshotPoller {
    snapshot: Snapshot,
    bus: bus::Bus,
}

impl SnapshotPoller {
    pub fn new(snapshot: Snapshot, bus: bus::Bus) -> Self {
        Self { snapshot, bus }
    }

    pub fn spawn(mut self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        let mut bus_listener = self.bus.receiver();
        let bus_sender = self.bus.sender();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = bus_listener.recv() => {
                        let reply_event = self.snapshot.event_process(event).unwrap();
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
