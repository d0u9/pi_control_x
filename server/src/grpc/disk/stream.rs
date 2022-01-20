use futures::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::stream::Stream;

use crate::grpc::grpc_api::WatchReply;
use tonic::Status;
use crate::bus_types::{BusRx, BusRxStream, BusData};

pub struct DiskWatchStream {
    inner: BusRxStream,
}

impl DiskWatchStream {
    pub fn new(bus_rx: BusRx) -> Self {
        Self { inner: BusRxStream::new(bus_rx) }
    }
}

impl Stream for DiskWatchStream {
    type Item = Result<WatchReply, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok((data, _saddr, _daddr)))) => {
                if let BusData::GrpcDisk(disk_data) = data {
                    let reply = WatchReply {
                        timestamp: disk_data.msg,
                    };
                    Poll::Ready(Some(Ok(reply)))
                } else {
                    Poll::Pending
                }
            }
            _ => Poll::Ready(None)
        }
    }
}


