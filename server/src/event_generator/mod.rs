use ::tokio::time::{self, Duration};
use crate::core::EventEnum;
use crate::shutdown::ShutdownReceiver;
use crate::core::bus::{self, BusSender, BusReceiver};
use crate::result::{Result, Error};

#[derive(Default)]
pub struct Builder {
    start: Duration,
    interval: Duration,
    event: Option<EventEnum>,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn event(mut self, event: EventEnum) -> Self {
        self.event = Some(event);
        self
    }

    pub fn start(mut self, start: Duration) -> Self {
        self.start = start;
        self
    }

    pub fn commit(self) -> Result<Generator> {
        let e1 = self.event.ok_or(Error::with_str("Generator event is not set"))?;
        let generator = Generator {
            start: self.start,
            interval: self.interval,
            event: e1,
        };
        Ok(generator)
    }
}

pub struct Generator {
    start: Duration,
    interval: Duration,
    event: EventEnum,
}

impl Generator {
    pub async fn issue_event(&self, first_issue: bool) -> Result<EventEnum> {
        if !first_issue {
            time::sleep(self.interval).await;
        }
        Ok(self.event.clone())
    }
}

pub struct GeneratorPoller {
    inner: Generator,
    bus: bus::Bus,
}

impl GeneratorPoller {
    pub fn new(generator: Generator, bus: bus::Bus) -> Self {
        Self {
            inner: generator,
            bus,
        }
    }

    pub fn spawn(mut self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        let bus_sender = self.bus.sender();
        let mut bus_listener = self.bus.receiver();
        let start = self.inner.start;

        tokio::spawn(async move {

            time::sleep(start).await;
            let first_event = self.inner.issue_event(true).await.unwrap();
            println!("First generator issue: {:?}", first_event);
            bus_sender.send(first_event);

            loop {
                tokio::select! {
                    // Listen bus, and print event if new message arrives at bus.
                    Ok(e) = bus_listener.recv() => {
                        println!("[BUS] event: {:?}", e);
                    }

                    // Send event to bus.
                    Ok(event) = self.inner.issue_event(false) => {
                        println!("Generator issue: {:?}", event);
                        bus_sender.send(event);
                    }

                    _ = shutdown.wait() => {
                        break;
                    }
                }
            }
        })
    }
}

