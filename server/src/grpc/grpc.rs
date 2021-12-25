use crate::core::bus;
use crate::core::EventEnum;
use crate::result::{Error, Result};
use ::std::future::Future;
use ::std::net::SocketAddr;
use ::tonic::transport::Server;

use super::disk::{DiskApiServer, EventDiskList};
use super::Event;

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

        let bus_switch = BusSwitch::new();

        let event_handler = EventHandler::new(bus_switch.clone());

        let inner = GrpcServerInner {
            addr: self.addr,
            disk_service,
            bus_switch,
        };

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
    bus_switch: BusSwitch,
}

pub(super) struct EventHandler {
    bus_switch: BusSwitch,
    bus: Option<bus::Bus<EventEnum>>,
}

impl GrpcServerInner {
    pub async fn serve(self, shutdown: impl Future<Output = ()>) -> Result<()> {
        let disk_service_bus = self.bus_switch.disk;
        let disk_service = self.disk_service.attach_bus(disk_service_bus).service();
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
    pub fn new(bus_switch: BusSwitch) -> Self {
        Self {
            bus_switch,
            bus: None,
        }
    }

    pub fn attach_bus(&mut self, bus: bus::Bus<EventEnum>) {
        self.bus = Some(bus)
    }

    pub fn event_process(&self, event: EventEnum) -> Result<Option<EventEnum>> {
        match event {
            _ => Ok(None),
        }
    }

    pub fn get_switch(&self) -> BusSwitch {
        self.bus_switch.clone()
    }
}

#[derive(Clone)]
pub(super) struct BusSwitch {
    disk: bus::Bus<EventDiskList>
}

impl BusSwitch {
    pub fn new() -> Self {
        Self {
            disk: bus::Bus::new(),
        }
    }

    pub async fn poll(&self) -> Option<EventEnum> {
        let mut disk = self.disk.receiver();
        let event = tokio::select! {
            Ok(e) = disk.recv() => { Event::Disk(e) }
        };

        Some(EventEnum::Grpc(event))
    }
}

