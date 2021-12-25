use crate::core::bus;
use crate::core::EventEnum;
use crate::shutdown::ShutdownReceiver;

pub struct Builder {
    cb: fn(EventEnum) -> Option<EventEnum>,
}

impl Builder {
    pub fn new() -> Self {
        Self { cb: |_| None }
    }

    pub fn event_process(mut self, cb: fn(EventEnum) -> Option<EventEnum>) -> Self {
        self.cb = cb;
        self
    }

    pub fn commit(self) -> Responder {
        Responder { cb: self.cb }
    }
}

pub struct Responder {
    cb: fn(EventEnum) -> Option<EventEnum>,
}

pub struct ResponderPoller {
    inner: Responder,
    bus: bus::Bus<EventEnum>,
}

impl ResponderPoller {
    pub fn new(responder: Responder, bus: bus::Bus<EventEnum>) -> Self {
        Self {
            inner: responder,
            bus,
        }
    }

    pub fn spawn(self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        let bus_sender = self.bus.sender();
        let mut bus_listener = self.bus.receiver();

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = bus_listener.recv() => {
                        println!("[BUS] event: {:?}", event);
                        if let Some(reply) = (self.inner.cb)(event) {
                            bus_sender.send(reply).unwrap();
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
