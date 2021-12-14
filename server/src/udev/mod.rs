#[cfg(test)]
mod mod_test;

pub(crate) mod udev;
pub(crate) use self::udev::UdevSocket;
pub(crate) use self::udev::UdevMonitor;

pub(crate) mod event;
pub(crate) use self::event::Event;

use ::std::sync::{Mutex, Arc};
use ::std::ffi::{OsStr, OsString};
use ::tokio::sync::broadcast;
use crate::result::{Result, Error};

struct UdevPoller {
    socket: Arc<Mutex<UdevSocket>>,
}

impl UdevPoller {
    pub fn new(socket: UdevSocket) -> Self {
        UdevPoller{ socket: Arc::new(Mutex::new(socket)) }
    }

    pub fn spawn(self) {
        tokio::spawn(async move {
            let mut socket = self.socket.lock().unwrap();
            let e = socket.read().await;
        });
    }
}
