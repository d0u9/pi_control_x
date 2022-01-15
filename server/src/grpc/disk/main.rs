use grpc_api::disk_server::{Disk, DiskServer};
use grpc_api::{ListReply, ListRequest};
use tonic::{Request, Response, Status};

use bus::wire::Endpoint;
use bus::address::Address;

mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}

#[derive(Debug, Clone, Default)]
pub struct DiskBusData {
    pub msg: String,
}

#[derive(Debug, Default)]
pub struct DiskApiService {
    bus_endpoint: Option<Endpoint<DiskBusData>>,
}

impl DiskApiService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn attach_bus(&mut self, endpoint: Endpoint<DiskBusData>) {
        self.bus_endpoint = Some(endpoint);
    }

    pub fn service(self) -> DiskServer<Self> {
        DiskServer::new(self)
    }
}

#[tonic::async_trait]
impl Disk for DiskApiService {
    async fn list(&self, request: Request<ListRequest>) -> Result<Response<ListReply>, Status> {
        let endpoint = self.bus_endpoint
                      .as_ref()
                      .ok_or_else(|| Status::failed_precondition("No internal bus attached"))?
                      .clone();

        let (tx, mut rx) = endpoint.split();
        tx.send(Address::new("disk_enumerator"), DiskBusData{ msg: "request request".to_string() });
        let data = rx.recv_data().await.unwrap();

        let request = request.into_inner();
        let reply = ListReply {
            // timestamp: format!("reply: {}", request.timestamp),
            timestamp: format!("reply: {}", data.msg),
        };
        Ok(Response::new(reply))
    }
}
