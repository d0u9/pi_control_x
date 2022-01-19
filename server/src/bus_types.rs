use tokio::time::Duration;

pub use bus::switch::SwitchError;
pub use bus::wire::EndpointError;
use bus::wire::Endpoint;
use bus::wire::{Tx, Rx};
use bus::switch::SwitchCtrl;
use bus::address::Address;

use crate::grpc::disk::DiskBusData;

#[derive(Clone, Debug)]
pub enum BusData {
    GrpcDisk(DiskBusData),
    UNSPEC,
}

impl BusData {
    pub fn grpc_disk(data: DiskBusData) -> Self {
        Self::GrpcDisk(data)
    }
}

#[derive(Debug)]
pub struct BusTx {
    inner: Tx<BusData>,
}

impl BusTx {
    pub fn send(&self, that_addr: Address, data: BusData) {
        self.inner.send(that_addr, data)
    }
}

#[derive(Debug)]
pub struct BusRx {
    inner: Rx<BusData>,
}

impl BusRx {
    pub async fn recv_data_timeout(&mut self, timeout: Duration) -> Result<BusData, EndpointError> {
        self.inner.recv_data_timeout(timeout).await
    }
}

#[derive(Clone, Debug)]
pub struct BusEndpoint {
    inner: Endpoint<BusData>,
}

impl BusEndpoint {
    pub fn split(self) -> (BusTx, BusRx) {
        let (tx, rx) = self.inner.split();
        (BusTx{inner: tx}, BusRx{inner: rx})
    }
}

#[derive(Clone, Debug)]
pub struct BusSwtichCtrl {
    inner: SwitchCtrl<BusData>,
}

impl BusSwtichCtrl {
    pub fn new(inner: SwitchCtrl<BusData>) -> Self {
        Self{ inner }
    }

    pub async fn add_endpoint(&mut self, this_addr: Address) -> Result<BusEndpoint, SwitchError> {
        let ep = self.inner.add_endpoint(this_addr).await?;
        Ok(BusEndpoint {
            inner: ep,
        })
    }
}


