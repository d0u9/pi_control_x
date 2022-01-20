use std::sync::Arc;

use crate::grpc::grpc_api::disk_server::{Disk, DiskServer};
use crate::grpc::grpc_api::{ListReply, ListRequest, WatchReply, WatchRequest};
use tokio::time::Duration;
use tonic::{Request, Response, Status};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::bus_types::{BusSwtichCtrl, BusEndpoint, BusData};

use super::stream::DiskWatchStream;
use super::super::error::{GrpcResult, GrpcError};

use bus::address::Address;

#[derive(Debug, Clone)]
pub struct DiskBusData {
    pub msg: String,
}

#[derive(Debug)]
pub struct DiskApiService {
    switch_ctrl: Arc<Mutex<BusSwtichCtrl>>,
    watch_endpoint: BusEndpoint,
}

impl DiskApiService {
    pub async fn new(mut switch_ctrl: BusSwtichCtrl) -> Self {
        let new_endpoint = switch_ctrl.add_endpoint(Address::new("grpc_disk_watch")).await;
        let watch_endpoint = match new_endpoint {
            Ok(ep) => ep,
            Err(_e) => panic!("create disk api service failed"),
        };

        Self {
            switch_ctrl: Arc::new(Mutex::new(switch_ctrl)),
            watch_endpoint,
        }
    }

    pub fn service(self) -> DiskServer<Self> {
        DiskServer::new(self)
    }

    pub async fn new_endpoint(&self, this_addr: Address) -> GrpcResult<BusEndpoint> {
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
        tx.send(that_addr, BusData::GrpcDisk(DiskBusData{
            msg: format!("request request {:?}", request.into_inner()),
        }));

        let data = rx.recv_data_timeout(Duration::from_secs(3)).await.map_err(|e| {
            GrpcError::bus_err(e)
        })?;

        let data = match data {
            BusData::GrpcDisk(data) => data,
            _ => { return Err(Status::internal("data not match")); },
        };

        let reply = ListReply {
            // timestamp: format!("reply: {}", request.timestamp),
            timestamp: format!("reply: {}", data.msg),
        };
        Ok(Response::new(reply))
    }

    /*
    type WatchStream = ReceiverStream<Result<WatchReply, Status>>;

    async fn watch(&self, _request: Request<WatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        let (tx, rx) = mpsc::channel::<Result<WatchReply, Status>>(4);

        tokio::spawn(async move {
            let replys = vec![WatchReply{ timestamp: String::from("heeeeello") }];
            for reply in replys {
                tx.send(Ok(reply)).await.unwrap();
            }

            println!(" /// done sending");
        });

        Ok(Response::new(ReceiverStream::new(rx)))
        // Err(Status::internal("eeeeeee"))
    }
    */





    // /*
    type WatchStream = DiskWatchStream;

    async fn watch(&self, _request: Request<WatchRequest>) -> Result<Response<Self::WatchStream>, Status> {
        let that_addr = Address::new("disk_enumerator");
        let disk_watch_endpoint = self.watch_endpoint.clone();
        let (tx, rx) = disk_watch_endpoint.split();


        tx.send(that_addr, BusData::GrpcDisk(DiskBusData{
            msg: format!("request request {:?}", "sdfs"),
        }));

        let disk_watch_stream = DiskWatchStream::new(rx);

        Ok(Response::new(disk_watch_stream))
    }
    // */
}


