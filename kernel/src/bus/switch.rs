use std::fmt::Debug;
use std::collections::HashMap;
use std::future::Future;

use log::trace;

use super::wire::{Rx, Tx, Endpoint, EndpointError};
use super::address::Address;

#[derive(Debug)]
pub enum SwitchError {
    AddressInUsed,
}

pub struct Builder<T> {
    endpoints: HashMap<Address, Endpoint<T>>
}

impl<T: Debug + Clone> Builder<T> {
    pub fn attach(mut self, addr: Address, endpoint: Endpoint<T>) -> Result<Self, SwitchError> {
        if self.endpoints.get(&addr).is_some() {
            return Err(SwitchError::AddressInUsed);
        }

        self.endpoints.insert(addr.clone(), endpoint);

        Ok(self)
    }

    pub fn done(self) -> Switch<T> {
        let (txs, rxs): (HashMap<Address, Tx<T>>, Vec<Rx<T>>) = self.endpoints.into_iter()
            .map(|(key, val)| {
                let (tx, rx) = val.split();
                ((key, tx), rx)
            })
            .unzip();

        Switch {
            txs,
            rxs,
        }
    }
}

pub struct Switch<T> {
    txs: HashMap<Address, Tx<T>>,
    rxs: Vec<Rx<T>>,
}

impl<T: Clone + Debug> Switch<T> {
    pub fn builder() -> Builder<T> {
        Builder {
            endpoints: HashMap::new(),
        }
    }

    async fn inner_poll(self) {
        let Self {
            mut rxs,
            ..
        } = self;

        let mut last_closed = Option::<usize>::None;
        loop {
            {
                let pin_futures = rxs.iter_mut().map(|rx| Box::pin(rx.recv())).collect::<Vec<_>>();
                match futures::future::select_all(pin_futures).await {
                    (Ok(pkt), _, _) => {
                        trace!("switch receives {:?}", pkt);
                    }
                    (Err(EndpointError::Closed), i, _) => {
                        last_closed = Some(i);
                    }
                };
            }
            if let Some(idx) = last_closed {
                rxs = rxs.into_iter().enumerate().filter(|(i, _)| *i != idx).map(|(_, val)| val).collect::<Vec<_>>();
            }
            last_closed = Option::<usize>::None;
        }
    }


    pub async fn poll(self, shutdown: impl Future<Output=()>) {
        tokio::select! {
            _ = shutdown => {
                trace!("switch receives shutdown signal");
            },
            _ = self.inner_poll() => { },
        }
    }
}
