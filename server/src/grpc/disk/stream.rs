use futures::future::FutureExt;
use futures::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use futures::stream::Stream;

use crate::grpc::grpc_api::{ListReply, ListRequest, WatchReply, WatchRequest};
use tokio::time::Duration;
use tonic::{Request, Response, Status};
use crate::bus_types::BusRx;

pub struct DiskWatchStream {
    inner: BusRx,
}

impl DiskWatchStream {
    pub fn new(bus_rx: BusRx) -> Self {
        Self { inner: bus_rx }
    }
}

impl Stream for DiskWatchStream {
    type Item = Result<WatchReply, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        println!("pppppoooooooolllllllllll");
        let mut x = Box::pin(self.inner.recv_data_timeout(Duration::from_secs(5)));

        let data = match x.poll_unpin(cx) {
            Poll::Pending => { println!("polling"); return Poll::Pending; },
            Poll::Ready(Ok(data)) => data,
            Poll::Ready(Err(_err)) => { println!("error"); return Poll::Ready(None); },
        };

        Poll::Ready(Some(Ok(WatchReply{ timestamp: format!("pppppppppp {:?}", data) })))

        /*
        let future = async {
            self.inner.recv_data_timeout(Duration::from_secs(5)).await
        };
        match Pin::new(&mut future).poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(data)) => Poll::Pending,
            Poll::Ready(Err(data)) => Poll::Pending,
        }
        */
    }
}


