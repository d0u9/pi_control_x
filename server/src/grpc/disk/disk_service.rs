use tonic::{Request, Response, Status};
use grpc_api::disk_server::{Disk, DiskServer};
use grpc_api::{ListRequest, ListReply};

use crate::core::bus;
use crate::core::EventEnum;

use super::super::Event;
use super::event::DiskListEvent;

pub mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct DiskApiServer {
    bus: Option<bus::Bus>,
}

impl DiskApiServer {
    pub fn new() -> Self {
        DiskApiServer::default()
    }

    pub fn service(self) -> DiskServer<Self> {
        self.bus.as_ref().expect("DiskApiServer has no bus attached");
        DiskServer::new(self)
    }

    pub fn attach_bus(mut self, bus: bus::Bus) -> Self {
        self.bus = Some(bus);
        self
    }
}

#[tonic::async_trait]
impl Disk for DiskApiServer {
    async fn list(
        &self,
        request: Request<ListRequest>,
    ) -> Result<Response<ListReply>, Status> {
        let bus_sender = self.bus.as_ref().unwrap().sender();
        let request = request.into_inner();
        let reply = ListReply {
            timestamp: format!("reply: {}", request.timestamp.clone()),
        };
        bus_sender.send(
            EventEnum::Grpc(
                Event::DiskList (
                    DiskListEvent {
                        timestamp: request.timestamp,
                    }
                )
            )
        ).unwrap();
        Ok(Response::new(reply))
    }
}


