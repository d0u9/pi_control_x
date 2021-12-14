use ::std::ffi::OsStr;
use ::udev::MonitorBuilder;
use ::tokio::sync::broadcast;
use ::mio::{Events, Interest, Poll, Token};

use crate::result::Result;
use super::event::Event;

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

    pub fn listen(self, tx: broadcast::Sender<Event>) -> Result<()> {
        let mut socket = self.builder.listen().unwrap();

        let mut poll = Poll::new()?;
        let mut events = Events::with_capacity(1024);

        poll.registry().register(
            &mut socket,
            Token(0),
            Interest::READABLE | Interest::WRITABLE,
        )?;

        loop {
            poll.poll(&mut events, None)?;

            for event in &events {
                if event.token() == Token(0) && event.is_writable() {
                    socket.clone().for_each(|x| print_event(x));
                }
            }
        }

        // Ok(())
    }
}

fn print_event(event: udev::Event) {
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

