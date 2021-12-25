#[cfg(test)]
mod mod_test;

pub mod grpc;
pub use grpc::{Builder, GrpcServer};

pub mod event;
pub use event::Event;

pub mod disk;

use crate::core::bus;
use crate::shutdown::{self, ShutdownReceiver};

pub struct GrpcPoller {
    server: GrpcServer,
    bus: bus::Bus,
}

impl GrpcPoller {
    pub fn new(server: GrpcServer, bus: bus::Bus) -> Self {
        Self { server, bus }
    }

    pub fn spawn(self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        let mut bus_listener = self.bus.receiver();
        let bus_sender = self.bus.sender();
        let server = self.server.server;
        let event_handler = self.server.event_handler;
        let (inner_shuttx, mut inner_shutrx) = shutdown::new();

        tokio::spawn(async move {
            server.serve(self.bus, inner_shutrx.wait()).await.unwrap();
        });

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Ok(event) = bus_listener.recv() => {
                        let reply_event = event_handler.event_process(event).unwrap();
                        if let Some(event) = reply_event {
                            let _ = bus_sender.send(event);
                        }
                    }
                    _ = shutdown.wait() => {
                        break;
                    }
                }
            }

            inner_shuttx.shutdown();
        })
    }
}
