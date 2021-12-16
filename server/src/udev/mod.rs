#[cfg(test)]
mod mod_test;

pub(crate) mod udev;
#[allow(unused_imports)]
pub(crate) use self::udev::UdevSocket;
#[allow(unused_imports)]
pub(crate) use self::udev::UdevMonitor;

pub(crate) mod event;
#[allow(unused_imports)]
pub(crate) use self::event::Event;

use crate::shutdown::ShutdownReceiver;
use crate::core::EventEnum;
use crate::core::bus::{self, BusSender, BusReceiver};

struct UdevPoller {
    socket: UdevSocket,
    notifier: BusSender,
}

impl UdevPoller {
    pub fn new(socket: UdevSocket, bus: bus::Bus) -> Self {
        Self {
            socket,
            notifier: bus.sender(),
        }
    }

    pub fn spawn(mut self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(events) = self.socket.read() => {
                        events.into_iter().for_each(|x| {
                            if let Err(e) = self.notifier.send(EventEnum::Udev(x)) {
                                println!("Lost event: error = {:?}", e);
                            }
                        });
                    }
                    _ = shutdown.wait() => {
                        break;
                    }
                }
            }
        })
    }
}
