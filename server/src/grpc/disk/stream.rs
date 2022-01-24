use futures::stream::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::bus_types::{BusData, BusRx, BusRxStream};
use crate::grpc::grpc_api::WatchReply as ApiWatchReply;
use tonic::Status;

pub struct DiskWatchStream {
    inner: BusRxStream,
}

impl DiskWatchStream {
    pub fn new(bus_rx: BusRx) -> Self {
        Self {
            inner: BusRxStream::new(bus_rx),
        }
    }
}

impl Stream for DiskWatchStream {
    type Item = Result<ApiWatchReply, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        match self.inner.poll_next_unpin(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(Ok((data, _saddr, _daddr)))) => {
                if let BusData::GrpcDisk(disk_data) = data {
                    let reply = ApiWatchReply {
                        timestamp: disk_data.msg,
                    };
                    Poll::Ready(Some(Ok(reply)))
                } else {
                    Poll::Pending
                }
            }
            _ => Poll::Ready(None),
        }
    }
}
