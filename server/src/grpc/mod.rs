#[cfg(test)]
mod mod_test;

pub mod grpc;
pub use grpc::{Builder, GrpcServer};

pub mod event;
pub use event::Event;

pub mod disk;


use crate::core::bus;
use crate::shutdown::ShutdownReceiver;

pub struct GrpcPoller {
    server: GrpcServer,
    bus: bus::Bus,
}

impl GrpcPoller {
    pub fn new(server: GrpcServer, bus: bus::Bus) -> Self {
        Self {
            server,
            bus,
        }
    }

    pub fn spawn(self, shutdown: ShutdownReceiver) -> tokio::task::JoinHandle<()> {
        let mut shutdown = shutdown;
        let mut bus_listener = self.bus.receiver();
        tokio::spawn(async move {
            self.server.serve(self.bus, shutdown.wait()).await.unwrap();
        })
    }
}
