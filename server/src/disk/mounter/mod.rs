#[cfg(test)]
mod mod_test;

pub(crate) mod event;
#[allow(unused_imports)]
pub(crate) use self::event::Event;

pub mod mounter;
pub use mounter::*;

use crate::core::bus::{self, BusReceiver, BusSender};
use crate::shutdown::ShutdownReceiver;

pub struct MounterPoller {
    mounter: Mounter,
    bus: bus::Bus,
}

impl MounterPoller {
    pub fn new(mounter: Mounter, bus: bus::Bus) -> Self {
        Self { mounter, bus }
    }

    pub fn spawn(self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        let mut bus_listener = self.bus.receiver();
        let bus_sender = self.bus.sender();
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = bus_listener.recv() => {
                        let reply_event = self.mounter.event_process(event).unwrap();
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
