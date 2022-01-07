use log::trace;
use std::convert::From;
use std::fmt::Debug;

use futures::Future;

use super::packet::Packet;
use super::wire::{Endpoint, Tx};

#[derive(Debug)]
pub struct Router<U, V> {
    ep0: Endpoint<U>,
    ep1: Endpoint<V>,
}

impl<U, V> Router<U, V>
where
    U: Clone + Debug + From<V>,
    V: Clone + Debug + From<U>,
{
    pub fn new(endpoint0: Endpoint<U>, endpoint1: Endpoint<V>) -> Self {
        Self {
            ep0: endpoint0,
            ep1: endpoint1,
        }
    }

    pub async fn poll(self, shutdown: impl Future<Output = ()>) {
        tokio::select! {
            _ = shutdown => {
                trace!("Router receives shutdown signal");
            }
            _ = self.inner_poll() => {
            }
        }
    }

    async fn inner_poll(self) {
        let (tx0, mut rx0) = self.ep0.split();
        let (tx1, mut rx1) = self.ep1.split();
        loop {
            tokio::select! {
                Ok(pkt) = rx0.recv() => {
                    Self::route(&tx1, pkt);
                }
                Ok(pkt) = rx1.recv() => {
                    Self::route(&tx0, pkt);
                }
            }
        }
    }

    fn route<F, T>(tx: &Tx<T>, pkt: Packet<F>)
    where
        F: Clone + Debug,
        T: Clone + Debug + From<F>,
    {
        tx.send_pkt(pkt.into());
    }
}
