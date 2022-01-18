use std::sync::Arc;

use grpc_api::disk_server::{Disk, DiskServer};
use grpc_api::{ListReply, ListRequest};
use tokio::time::Duration;
use tonic::{Request, Response, Status};
use tokio::sync::Mutex;
use uuid::Uuid;

use super::super::error::{GrpcResult, GrpcError};

use bus::wire::Endpoint;
use bus::address::Address;
use bus::switch::SwitchCtrl;

mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}

#[derive(Debug, Clone)]
pub struct DiskBusData {
    pub msg: String,
}

#[derive(Debug)]
pub struct DiskApiService {
    switch_ctrl: Arc<Mutex<SwitchCtrl<DiskBusData>>>
}

impl DiskApiService {
    pub fn new(switch_ctrl: SwitchCtrl<DiskBusData>) -> Self {
        Self {
            switch_ctrl: Arc::new(Mutex::new(switch_ctrl)),
        }
    }

    pub fn service(self) -> DiskServer<Self> {
        DiskServer::new(self)
    }

    pub async fn new_endpoint(&self, this_addr: Address) -> GrpcResult<Endpoint<DiskBusData>> {
        let mut switch_ctrl_lock = self.switch_ctrl.lock().await;
        match switch_ctrl_lock.add_endpoint(this_addr).await {
            Ok(ep) => Ok(ep),
            Err(e) => Err(GrpcError::bus_err(e)),
        }
    }
}

#[tonic::async_trait]
impl Disk for DiskApiService {
    async fn list(&self, request: Request<ListRequest>) -> Result<Response<ListReply>, Status> {
        let uuid = Uuid::new_v4().to_string();
        let this_addr = Address::new(&uuid);
        let that_addr = Address::new("disk_enumerator");
        let endpoint = self.new_endpoint(this_addr).await?;

        let (tx, mut rx) = endpoint.split();
        tx.send(that_addr, DiskBusData{
            msg: format!("request request {:?}", request.into_inner()),
        });

        let data = match rx.recv_data_timeout(Duration::from_secs(3)).await {
            Err(e) => return Err(Status::cancelled(format!("{:?}", e))),
            Ok(data) => data,
        };

        let reply = ListReply {
            // timestamp: format!("reply: {}", request.timestamp),
            timestamp: format!("reply: {}", data.msg),
        };
        Ok(Response::new(reply))
    }
}
