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

use ::tokio::sync::broadcast;
use crate::Shutdown::ShutdownReceiver;

struct UdevPoller {
    socket: UdevSocket,
    notifier: broadcast::Sender<Event>,
}

impl UdevPoller {
    pub fn new(socket: UdevSocket) -> Self {
        let (tx, _rx) = broadcast::channel(16);
        UdevPoller{
            socket,
            notifier: tx,
        }
    }

    pub fn spawn(mut self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(events) = self.socket.read() => {
                        events.into_iter().for_each(|x| {
                            if let Err(e) = self.notifier.send(x) {
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

    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.notifier.subscribe()
    }
}
