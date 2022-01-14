use grpc_api::disk_server::{Disk, DiskServer};
use grpc_api::{ListReply, ListRequest};
use tonic::{Request, Response, Status};

mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct DiskApiService {
}

impl DiskApiService {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn service(self) -> DiskServer<Self> {
        DiskServer::new(self)
    }
}

#[tonic::async_trait]
impl Disk for DiskApiService {
    async fn list(&self, request: Request<ListRequest>) -> Result<Response<ListReply>, Status> {
        let request = request.into_inner();
        let reply = ListReply {
            timestamp: format!("reply: {}", request.timestamp),
        };
        Ok(Response::new(reply))
    }
}
