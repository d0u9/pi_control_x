use std::convert::TryFrom;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tokyo_bus::address::Address;

use crate::grpc::grpc_api::disk_server::{Disk as ApiDisk, DiskServer as ApiDiskServer};
use crate::grpc::grpc_api::{
    ListReply as ApiListReply, ListRequest as ApiListRequest, WatchReply as ApiWatchReply,
    WatchRequest as ApiWatchRequest,
};
use tonic::{Request, Response, Status};

use crate::bus_types::SwitchCtrl;
use crate::bus_types::{BusData, BusEndpoint, BusRx, BusTx};

use super::super::error::{GrpcError, GrpcResult};
use super::stream::*;

#[derive(Debug, Clone)]
pub struct GrpcDiskData {
    pub msg: String,
}

impl TryFrom<GrpcDiskData> for ApiWatchReply {
    type Error = GrpcError;

    fn try_from(value: GrpcDiskData) -> Result<Self, Self::Error> {
        let replys = ApiWatchReply {
            timestamp: value.msg,
        };
        Ok(replys)
    }
}

#[derive(Debug)]
pub struct GrpcDiskApiService {
    switch_ctrl: SwitchCtrl,
    disk_watch_endpoint: Option<BusEndpoint>,
}

impl GrpcDiskApiService {
    pub fn new(switch_ctrl: SwitchCtrl) -> Self {
        Self {
            switch_ctrl,
            disk_watch_endpoint: None,
        }
    }

    pub async fn service(mut self) -> GrpcResult<ApiDiskServer<Self>> {
        let disk_watch_endpoint = self.new_endpoint(&Address::new("grpc-disk-watch")).await?;
        self.disk_watch_endpoint = Some(disk_watch_endpoint);
        Ok(ApiDiskServer::new(self))
    }

    fn get_disk_watch_endpoint(&self) -> BusEndpoint {
        self.disk_watch_endpoint.as_ref().unwrap().clone()
    }

    async fn new_endpoint(&self, addr: &Address) -> GrpcResult<BusEndpoint> {
        let ep = self.switch_ctrl.new_endpoint(addr).await?;
        Ok(ep)
    }

    async fn new_endpoint_pair(&self, addr: &Address) -> GrpcResult<(BusTx, BusRx)> {
        let endpoint = self.new_endpoint(addr).await?;
        let ret = endpoint.split()?;
        Ok(ret)
    }

    async fn list(
        &self,
        tx: &BusTx,
        rx: &mut BusRx,
        request: &str,
    ) -> Result<GrpcDiskData, Status> {
        let that_addr = Address::new("disk_enumerator");

        let request = BusData::GrpcDisk(GrpcDiskData {
            msg: format!("request request {:?}", request),
        });
        tx.send(&that_addr, request).unwrap();

        // wait response
        let data = match rx.recv().await {
            Ok((BusData::GrpcDisk(data), _, _)) => Ok(data),
            Err(e) => Err(GrpcError::from(e)),
            _ => Err(GrpcError::internal("Grpc disk list service: type mismatch")),
        }?;

        Ok(data)
    }
}

#[tonic::async_trait]
impl ApiDisk for GrpcDiskApiService {
    async fn list(
        &self,
        request: Request<ApiListRequest>,
    ) -> Result<Response<ApiListReply>, Status> {
        let this_addr = Address::random();
        let (tx, mut rx) = self.new_endpoint_pair(&this_addr).await?;

        let data = self
            .list(&tx, &mut rx, &format!("{:?}", request.into_inner()))
            .await?;

        let reply = ApiListReply {
            timestamp: format!("reply: {}", data.msg),
        };

        Ok(Response::new(reply))
    }

    type WatchStream = ReceiverStream<Result<ApiWatchReply, Status>>;

    async fn watch(
        &self,
        _request: Request<ApiWatchRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        let oneshot_data = {
            // Once the client connected, send it fresh data made from a oneshot query.
            let this_addr = Address::random();
            let (tx, mut rx) = self.new_endpoint_pair(&this_addr).await?;
            let data = self.list(&tx, &mut rx, &format!("{:?}", 256)).await?;
            ApiWatchReply::try_from(data)
        }?;

        let switch_endpoint = self.get_disk_watch_endpoint();
        let (_, switch_rx) = switch_endpoint.split().map_err(GrpcError::from)?;
        let (tx, rx) = mpsc::channel::<Result<ApiWatchReply, Status>>(2);

        tokio::spawn(async move {
            // TODO: unwrap
            tx.send(Ok(oneshot_data)).await.unwrap();

            let mut stream = DiskWatchStream::new(switch_rx);
            while let Some(stream) = stream.next().await {
                if let Ok(watch_reply) = stream {
                    // TODO: unwrap
                    tx.send(Ok(watch_reply)).await.unwrap();
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
