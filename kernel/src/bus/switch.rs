use std::fmt::Debug;
use std::collections::HashMap;
use std::future::Future;
use std::default::Default;

use log::trace;

use super::wire::{Rx, Tx, Endpoint};
use super::address::Address;
use super::packet::Packet;

#[derive(Debug)]
pub enum SwitchError {
    AddressInvalid,
    AddressInUsed,
}

pub struct Builder<T> {
    endpoints: HashMap<Address, Endpoint<T>>
}

impl<T: Debug + Clone> Builder<T> {
    pub fn attach(mut self, addr: Address, endpoint: Endpoint<T>) -> Result<Self, SwitchError> {
        if let Address::Broadcast = addr {
            return Err(SwitchError::AddressInvalid);
        }

        if self.endpoints.get(&addr).is_some() {
            return Err(SwitchError::AddressInUsed);
        }

        self.endpoints.insert(addr, endpoint);

        Ok(self)
    }

    pub fn done(self) -> Switch<T> {
        let ports = self.endpoints.into_iter()
            .map(|(addr, endpoint)| {
                let (tx, rx) = endpoint.split();
                let port = Port {
                    addr: addr.clone(),
                    tx,
                    rx,
                };
                (addr, port)
            }).collect::<HashMap<_, _>>();

        Switch {
            ports,
        }
    }
}

enum PollResult<T> {
    Packet(Packet<T>),
    Closed,
}


#[derive(Debug)]
struct Port<T> {
    addr: Address,
    rx: Rx<T>,
    tx: Tx<T>,
}

impl<T: Clone + Debug> Port<T> {
    async fn poll(&mut self) -> (&Self, PollResult<T>) {
        let result = match self.rx.recv().await {
            Ok(v) => { PollResult::Packet(v) }
            Err(_) => { PollResult::Closed }
        };

        (self, result)
    }

    fn addr(&self) -> Address {
        self.addr.clone()
    }

    fn send(&self, val: Packet<T>) {
        self.tx.send_pkt(val);
    }
}

pub struct Switch<T> {
    ports: HashMap<Address, Port<T>>,
}

impl<T: Clone + Debug> Switch<T> {
    pub fn builder() -> Builder<T> {
        Builder {
            endpoints: HashMap::new(),
        }
    }

    async fn inner_poll(mut self) {
        loop {
            let (ready_port, poll_result) = {
                let pin_futures = self.ports.iter_mut().map(|(_, port)| Box::pin(port.poll()));
                match futures::future::select_all(pin_futures).await {
                    ((port, result), _, _) => {
                        (port, result)
                    }
                }
            };

            let ready_addr = ready_port.addr();
            if let PollResult::Packet(mut pkt) = poll_result {
                trace!("New data arrivas at port ({}): {:?}", ready_addr, pkt);
                // Process received packet
                pkt.set_saddr(ready_addr.clone());
                self.process_pkt(pkt);
            } else {
                trace!("Port ({}) is closed", ready_addr);
                self.ports.remove(&ready_addr);
            }
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

    fn process_pkt(&self, pkt: Packet<T>) {
        let saddr = match pkt.ref_saddr() {
            Some(saddr) => saddr.to_owned(),
            None => {
                trace!("[Bug] pkt has no saddr: {:?}", pkt);
                return ();
            }
        };

        let daddr = pkt.ref_daddr();
        if let Address::Broadcast = daddr {
            trace!("Braodcast pkt: {:?}", pkt);
            self.ports.iter()
                .filter(|(addr, _)| *addr != &saddr)
                .map(|(_, port)| port)
                .for_each(|port| {
                    port.send(pkt.clone());
                });

            return ();
        }

        let peer = self.ports.get(daddr);
        match peer {
            Some(peer) => peer.send(pkt),
            None => {
                trace!("Cannot find addr({}) in local, drop", daddr);
                trace!("current ports: {:?}", self.ports);
                return ();
            }
        }
    }
}
