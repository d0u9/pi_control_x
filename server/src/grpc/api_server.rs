use tonic::{transport::Server, Request, Response, Status};
use grpc_api::disk_server::{Disk, DiskServer};
use grpc_api::{ListRequest, ListReply};

pub mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}

#[derive(Debug, Default)]
pub struct DiskApiServer {

}

impl DiskApiServer {
    pub fn new() -> Self {
        DiskApiServer::default()
    }

    pub fn service(self) -> DiskServer<Self> {
        DiskServer::new(self)
    }
}

#[tonic::async_trait]
impl Disk for DiskApiServer {
    async fn list(
        &self,
        request: Request<ListRequest>,
    ) -> Result<Response<ListReply>, Status> {
        let reply = ListReply {
            timestamp: request.into_inner().timestamp.clone(),
        };
        Ok(Response::new(reply))
    }

}


