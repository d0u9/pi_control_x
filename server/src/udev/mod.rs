#[cfg(test)]
mod mod_test;

pub(crate) mod udev;
pub(crate) use self::udev::Udev;

pub(crate) mod event;
pub(crate) use self::event::Event;

use ::std::ffi::{OsStr, OsString};
use ::tokio::sync::broadcast;
use crate::result::{Result, Error};

struct ThreadUdev {
    matcher: Vec<(OsString, OsString)>,
}

impl ThreadUdev {
    pub fn new() -> Result<ThreadUdev> {
        Ok(ThreadUdev{
            matcher: Vec::new(),
        })
    }

    pub fn match_subsystem_devtype<T, U>(mut self, subsystem: T, devtype: U) -> Result<Self>
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>,
    {
        self.matcher.push((subsystem.as_ref().to_os_string(), devtype.as_ref().to_os_string()));
        Ok(self)
    }

    fn spawn_run(self) -> Result<broadcast::Receiver<Event>> {
        let (tx, rx) = broadcast::channel(16);

        tokio::spawn(async move {
            let matcher = self.matcher;
            let mut udev = Udev::new().unwrap();

            for mat in matcher.into_iter() {
                udev = udev.match_subsystem_devtype(mat.0, mat.1).unwrap();
            }

            udev.listen(tx);
        });

        Ok(rx)
    }
}
