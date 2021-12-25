use crate::core::bus;
use crate::core::EventEnum;
use crate::result::{Error, Result};
use ::std::future::Future;
use ::std::net::SocketAddr;
use ::tonic::transport::Server;

use super::disk::DiskApiServer;

#[derive(Debug, Default)]
pub struct Builder {
    addr: Option<SocketAddr>,
}

impl Builder {
    pub fn new() -> Self {
        Builder::default()
    }

    pub fn address(mut self, addr: &str) -> Result<Self> {
        let addr = addr.parse()?;
        self.addr = Some(addr);
        Ok(self)
    }

    pub fn commit(self) -> Result<GrpcServer> {
        self.addr.ok_or(Error::with_str("no address assigned"))?;
        let disk_service = DiskApiServer::new();

        let inner = GrpcServerInner {
            addr: self.addr,
            disk_service,
        };

        let event_handler = EventHandler;

        let grpc_server = GrpcServer {
            server: inner,
            event_handler,
        };

        Ok(grpc_server)
    }
}

pub struct GrpcServer {
    pub(super) server: GrpcServerInner,
    pub(super) event_handler: EventHandler,
}

pub(super) struct GrpcServerInner {
    addr: Option<SocketAddr>,
    disk_service: DiskApiServer,
}

pub(super) struct EventHandler;

impl GrpcServerInner {
    pub async fn serve(self, bus: bus::Bus, shutdown: impl Future<Output = ()>) -> Result<()> {
        let disk_service = self.disk_service.attach_bus(bus).service();
        let addr = self.addr.unwrap();

        println!("GRPC server is listening on: {}", &self.addr.unwrap());

        Server::builder()
            .add_service(disk_service)
            .serve_with_shutdown(addr, shutdown)
            .await
            .unwrap();

        Ok(())
    }
}

impl EventHandler {
    pub fn event_process(&self, event: EventEnum) -> Result<Option<EventEnum>> {
        match event {
            _ => Ok(None),
        }
    }
}
