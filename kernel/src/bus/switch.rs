use std::collections::HashMap;
use std::default::Default;
use std::fmt::Debug;
use std::future::Future;

use log::trace;

use super::address::Address;
use super::packet::{Packet, BusPacket, LastHop};
use super::wire::{Endpoint, Rx, Tx};

#[derive(Debug)]
pub enum SwitchError {
    AddressInvalid,
    AddressInUsed,
}

pub struct Builder<T> {
    // bool = true represents a router
    endpoints: HashMap<Address, (Endpoint<T>, bool)>,
    gateway: Option<Address>,
}

impl<T: Debug + Clone> Builder<T> {
    pub fn attach(self, addr: Address, endpoint: Endpoint<T>) -> Result<Self, SwitchError> {
        self.attach_endpoint(addr, endpoint, false)
    }

    pub fn attach_router(self, addr: Address, endpoint: Endpoint<T>) -> Result<Self, SwitchError> {
        self.attach_endpoint(addr, endpoint, true)
    }

    pub fn set_gateway(mut self, gateway: Address) -> Result<Self, SwitchError> {
        let (_, is_router) = self.endpoints.get(&gateway).ok_or(SwitchError::AddressInvalid)?;
        if !(*is_router) {
            Err(SwitchError::AddressInvalid)
        } else {
            self.gateway = Some(gateway);
            Ok(self)
        }
    }

    pub fn done(self) -> Switch<T> {
        let router_addrs = self.endpoints.iter()
            .map(|(addr, _)| addr.clone())
            .collect::<_>();

        let ports = self
            .endpoints
            .into_iter()
            .map(|(addr, endpoint)| {
                let (tx, rx) = endpoint.0.split();
                let port = Port {
                    is_router: endpoint.1,
                    addr: addr.clone(),
                    tx,
                    rx,
                };
                (addr, port)
            })
            .collect::<HashMap<_, _>>();

        Switch {
            ports,
            router_addrs,
            gateway: self.gateway,
        }
    }

    fn attach_endpoint(mut self, addr: Address, endpoint: Endpoint<T>, is_router: bool) -> Result<Self, SwitchError> {
        if let Address::Broadcast = addr {
            return Err(SwitchError::AddressInvalid);
        }

        if self.endpoints.get(&addr).is_some() {
            return Err(SwitchError::AddressInUsed);
        }

        self.endpoints.insert(addr, (endpoint, is_router));

        Ok(self)
    }
}

enum PollResult<T> {
    Packet(Packet<T>),
    Closed,
}

#[derive(Debug)]
struct Port<T> {
    is_router: bool,
    addr: Address,
    rx: Rx<T>,
    tx: Tx<T>,
}

impl<T: Clone + Debug> Port<T> {
    async fn poll(&mut self) -> (&Self, PollResult<T>) {
        let result = match self.rx.recv().await {
            Ok(v) => PollResult::Packet(v),
            Err(_) => PollResult::Closed,
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
    router_addrs: Vec<Address>,
    gateway: Option<Address>,
}

impl<T: Clone + Debug> Switch<T> {
    pub fn builder() -> Builder<T> {
        Builder {
            endpoints: HashMap::new(),
            gateway: None,
        }
    }

    pub async fn poll(self, shutdown: impl Future<Output = ()>) {
        tokio::select! {
            _ = shutdown => {
                trace!("switch receives shutdown signal");
            },
            _ = self.inner_poll() => { },
        }
    }

    async fn inner_poll(mut self) {
        loop {
            let (ready_port, poll_result) = {
                let pin_futures = self.ports.iter_mut().map(|(_, port)| Box::pin(port.poll()));
                let ((port, result), _, _) = futures::future::select_all(pin_futures).await;
                (port, result)
            };

            let ready_addr = ready_port.addr();
            if let PollResult::Packet(mut pkt) = poll_result {
                trace!("New data arrivas at port ({}): {:?}", ready_addr, pkt);
                // Process received packet
                pkt.set_saddr(ready_addr.clone());
                // convert local packet to bus packet
                let pkt = BusPacket::from_local_packet(pkt); 
                self.switch(&ready_addr, pkt);
                // self.process_pkt(pkt);
            } else {
                trace!("Port ({}) is closed", ready_addr);
                self.ports.remove(&ready_addr);
            }
        }
    }

    fn switch(&self, saddr: &Address, pkt: BusPacket<T>)  {
        let ref_inner = pkt.ref_inner();
        let ref_daddr = ref_inner.ref_daddr();

        match ref_daddr {
            Address::Broadcast => {
                self.broadcast(saddr, pkt);
            }
            Address::Addr(_) => {
                self.send_to(saddr, pkt);
            }
        }
    }

    fn broadcast(&self, saddr: &Address, pkt: BusPacket<T>) {
        trace!("Braodcast pkt: {:?}", pkt);
        let local_pkt = pkt.into_local_packet();
        self.ports
            .iter()
            .filter(|(addr, _)| addr != &saddr)
            .map(|(_, port)| port)
            .for_each(|port| {
                port.send(local_pkt.clone());
        });
    }

    fn send_to(&self, _saddr: &Address, pkt: BusPacket<T>) {
        let ref_inner = pkt.ref_inner();
        let ref_daddr = ref_inner.ref_daddr();

        let dst_port = self.ports.get(ref_daddr);

        if let Some(port) = dst_port {
            let local_pkt = pkt.into_local_packet();
            port.send(local_pkt);
        } else {
            // route or drop
            self.route_to(pkt);
        }
    }

    fn route_to(&self, pkt: BusPacket<T>) {
        let ref_last_hop = pkt.ref_last_hop();
        let candidates = match ref_last_hop {
            LastHop::Local => {
                // default gateway is used if packet is sent from local
                match self.gateway {
                    None => {
                        trace!("Packet is not sent to local, and not gateway is specificed, drop");
                        return;
                    },
                    Some(ref gateway) => {
                        vec![gateway]
                    }
                }
            },
            LastHop::Router(last_router_addr) => {
                // Packet is sent from another router and its daddr is not in local
                // Redirect to all routers
                self.router_addrs.iter()
                    .filter(|addr| *addr != last_router_addr)
                    .collect::<Vec<_>>()
            },
        };

        let local_pkt = pkt.into_local_packet();
        candidates.into_iter().for_each(|addr| {
            let port = self.ports.get(addr);
            match port {
                None => {
                    trace!("BUG: route addr is invalid");
                },
                Some(port) => {
                    port.send(local_pkt.clone());
                }
            }
        });
    }
}
