use ::std::ffi::OsStr;
use ::udev::{MonitorBuilder, MonitorSocket};
use ::tokio::sync::broadcast;
use ::tokio::io::unix::AsyncFd;
use ::tokio::io::Interest;
// use ::mio::{Events, Interest, Poll, Token};

use crate::result::Result;
use super::event::{Event, Events};

pub(crate) struct Udev {
    builder: MonitorBuilder,
}

impl Udev {
    pub fn new() -> Result<Self> {
        let builder = udev::MonitorBuilder::new()?;

        Ok( Udev{ builder } )
    }

    pub fn match_subsystem_devtype<T, U>(self, subsystem: T, devtype: U) -> Result<Self>
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>,
    {
        let builder = self.builder
            .match_subsystem_devtype(subsystem, devtype)?;

        Ok( Udev{ builder } )
    }

    pub fn listen(self) -> Result<UdevSocket> {
        let mut socket = self.builder.listen()?;
        let mut socket = AsyncFd::with_interest(socket, Interest::READABLE)?;
        Ok(UdevSocket{ inner: socket })

    }
}

pub(crate) struct UdevSocket {
    inner: AsyncFd<MonitorSocket>,
}

impl UdevSocket {
    pub async fn read(&mut self) -> Result<Vec<Event>> {
        loop {
            let mut guard = self.inner.readable().await?;
            guard.clear_ready();

            let socket = self.inner.get_ref().clone();
            let events = socket.map(|x| x.into()).collect::<Vec<_>>();
            if events.len() > 0 {
                return Ok(events);
            } else {
                continue;
            }
        }
    }
}

fn print_event(event: &udev::Event) {
    println!(
        "{}: {} {} (subsystem={}, sysname={}, devtype={})",
        event.sequence_number(),
        event.event_type(),
        event.syspath().to_str().unwrap_or("---"),
        event
            .subsystem()
            .map_or("", |s| { s.to_str().unwrap_or("") }),
        event.sysname().to_str().unwrap_or(""),
        event.devtype().map_or("", |s| { s.to_str().unwrap_or("") })
    );
}

