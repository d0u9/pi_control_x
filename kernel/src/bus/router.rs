use std::convert::From;
use std::fmt::Debug;

use futures::Future;
use log::trace;
use uuid::Uuid;

use super::packet::Packet;
use super::wire::{Endpoint, Tx};

#[derive(Debug)]
pub enum RouterError {
    BuildError,
}
pub struct Builder<U, V> {
    name: String,
    ep0: Option<Endpoint<U>>,
    ep1: Option<Endpoint<V>>,
}

impl<U, V> Builder<U, V>
where
    U: Clone + Debug + From<V>,
    V: Clone + Debug + From<U>,
{
    pub fn set_name(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn set_endpoint0(mut self, endpoint: Endpoint<U>) -> Self {
        self.ep0 = Some(endpoint);
        self
    }

    pub fn set_endpoint1(mut self, endpoint: Endpoint<V>) -> Self {
        self.ep1 = Some(endpoint);
        self
    }

    pub fn done(self) -> Result<Router<U, V>, RouterError> {
        let ep0 = self.ep0.ok_or(RouterError::BuildError)?;
        let ep1 = self.ep1.ok_or(RouterError::BuildError)?;

        let router = Router {
            uuid: Uuid::new_v4(),
            name: self.name,
            ep0,
            ep1,
        };

        Ok(router)
    }
}

#[derive(Debug)]
pub struct Router<U, V> {
    uuid: Uuid,
    name: String,
    ep0: Endpoint<U>,
    ep1: Endpoint<V>,
}

impl<U, V> Router<U, V>
where
    U: Clone + Debug + From<V>,
    V: Clone + Debug + From<U>,
{
    pub fn builder() -> Builder<U, V> {
        Builder {
            name: "".to_string(),
            ep0: None,
            ep1: None,
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
        trace!("[Router({})] Start polling...", self.uuid);
        loop {
            tokio::select! {
                Ok(pkt) = rx0.recv() => {
                    trace!("[Router({})] Endpoint0 receives pkt: {:?}", self.uuid, pkt);
                    Self::route(&tx1, pkt);
                }
                Ok(pkt) = rx1.recv() => {
                    trace!("[Router({})] Endpoint1 receives pkt: {:?}", self.uuid, pkt);
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
