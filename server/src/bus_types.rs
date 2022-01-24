use std::convert::From;
use std::pin::Pin;
use std::task::{Context, Poll};

use futures::StreamExt;
use tokio_stream::Stream;

use tokyo_bus::address::Address;
use tokyo_bus::packet_endpoint::{
    PktEndpoint, PktEndpointErrKind, PktEndpointError, PktRx, PktRxStream, PktTx,
};
use tokyo_bus::switch::{SwitchErrKind, SwitchError, SwitchHandler};

use crate::grpc::lib::GrpcDiskData;

#[derive(Debug, Clone, Copy)]
pub enum BusErrKind {
    SwitchErr(SwitchErrKind),
    PktEndpointErr(PktEndpointErrKind),
}

#[derive(Debug, Clone)]
pub struct BusError {
    kind: BusErrKind,
    msg: String,
}

impl BusError {
    pub fn err_kind(&self) -> BusErrKind {
        self.kind
    }

    pub fn err_msg(&self) -> &str {
        &self.msg
    }
}

impl From<SwitchError> for BusError {
    fn from(err: SwitchError) -> Self {
        Self {
            kind: BusErrKind::SwitchErr(err.err_kind()),
            msg: format!("Switch Err: {:?}", err.err_msg()),
        }
    }
}

impl From<PktEndpointError> for BusError {
    fn from(err: PktEndpointError) -> Self {
        Self {
            kind: BusErrKind::PktEndpointErr(err.err_kind()),
            msg: format!("Switch Err: {:?}", err.err_msg()),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SwitchCtrl {
    inner: SwitchHandler<BusData>,
}

impl SwitchCtrl {
    pub fn new(switch_handler: SwitchHandler<BusData>) -> Self {
        Self {
            inner: switch_handler,
        }
    }

    pub async fn new_endpoint(&self, addr: &Address) -> Result<BusEndpoint, BusError> {
        let inner = self.inner.new_endpoint(addr.to_owned()).await?;
        Ok(BusEndpoint { inner })
    }
}

#[derive(Debug)]
pub struct BusTx {
    inner: PktTx<BusData>,
}

impl BusTx {
    pub fn send(&self, addr: &Address, val: BusData) -> Result<(), BusError> {
        let _ = self.inner.send_data(addr, val)?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct BusRx {
    inner: PktRx<BusData>,
}

impl BusRx {
    pub async fn recv(&mut self) -> Result<(BusData, Address, Address), BusError> {
        let val = self.inner.recv_tuple().await?;
        Ok(val)
    }
}

#[derive(Debug)]
pub struct BusRxStream {
    inner: PktRxStream<BusData>,
}

impl BusRxStream {
    pub fn new(rx: BusRx) -> Self {
        Self {
            inner: PktRxStream::new(rx.inner),
        }
    }
}

impl Stream for BusRxStream {
    type Item = Result<(BusData, Address, Address), BusError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok(pkt))) => Poll::Ready(Some(Ok(pkt.into_tuple()))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e.into()))),
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}

#[derive(Debug, Clone)]
pub struct BusEndpoint {
    inner: PktEndpoint<BusData>,
}

impl BusEndpoint {
    pub fn split(self) -> Result<(BusTx, BusRx), BusError> {
        let (tx, rx) = self.inner.split()?;
        Ok((BusTx { inner: tx }, BusRx { inner: rx }))
    }
}

#[derive(Debug, Clone)]
pub enum BusData {
    GrpcDisk(GrpcDiskData),
    Unspec,
}
