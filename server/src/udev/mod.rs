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

use crate::Shutdown::ShutdownReceiver;

struct UdevPoller {
    socket: UdevSocket,
}

impl UdevPoller {
    pub fn new(socket: UdevSocket) -> Self {
        UdevPoller{ socket }
    }

    pub fn spawn(mut self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    e = self.socket.read() => {
                        println!("Event: {:?}", e);
                    }
                    _ = shutdown.wait() => {
                        break;
                    }
                }
            }
        })
    }
}
