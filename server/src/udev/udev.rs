use ::std::ffi::OsStr;
use ::udev::{MonitorBuilder, MonitorSocket};
use ::tokio::io::unix::AsyncFd;
use ::tokio::io::Interest;
use ::std::os::unix::io::{AsRawFd, RawFd};
// use ::mio::{Events, Interest, Poll, Token};

use crate::result::Result;
use super::event::Event;

pub(crate) struct UdevMonitor {
    builder: MonitorBuilder,
}

impl UdevMonitor {
    pub fn new() -> Result<Self> {
        let builder = udev::MonitorBuilder::new()?;

        Ok( UdevMonitor{ builder } )
    }

    pub fn match_subsystem_devtype<T, U>(self, subsystem: T, devtype: U) -> Result<Self>
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>,
    {
        let builder = self.builder
            .match_subsystem_devtype(subsystem, devtype)?;

        Ok( UdevMonitor{ builder } )
    }

    pub fn listen(self) -> Result<UdevSocket> {
        let monitor_socket = self.builder.listen()?;
        let sock_fd = monitor_socket.as_raw_fd();
        let async_fd = AsyncFd::with_interest(sock_fd, Interest::READABLE)?;
        Ok(UdevSocket{ async_fd, monitor_socket })
    }
}

type MonitorSocketFd = RawFd;

pub(crate) struct UdevSocket {
    async_fd: AsyncFd<MonitorSocketFd>,
    monitor_socket: MonitorSocket,
}

impl UdevSocket {
    pub async fn read(&mut self) -> Result<Vec<Event>> {
        loop {
            let mut guard = self.async_fd.readable().await?;
            guard.clear_ready();

            let socket = self.monitor_socket.clone();
            let events = socket.map(|x| x.into()).collect::<Vec<_>>();
            if events.len() > 0 {
                return Ok(events);
            } else {
                continue;
            }
        }
    }
}
