use std::collections::HashMap;
use std::default::Default;
use std::fmt::Debug;
use std::future::Future;

use log::trace;
use uuid::Uuid;

use super::address::Address;
use super::packet::Packet;
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
    name: String,
}

impl<T: Debug + Clone> Builder<T> {
    pub fn attach(self, addr: Address, endpoint: Endpoint<T>) -> Result<Self, SwitchError> {
        self.attach_endpoint(addr, endpoint, false)
    }

    pub fn attach_router(self, addr: Address, endpoint: Endpoint<T>) -> Result<Self, SwitchError> {
        self.attach_endpoint(addr, endpoint, true)
    }

    pub fn set_gateway(mut self, gateway: Address) -> Result<Self, SwitchError> {
        let (_, is_router) = self
            .endpoints
            .get(&gateway)
            .ok_or(SwitchError::AddressInvalid)?;
        if !(*is_router) {
            Err(SwitchError::AddressInvalid)
        } else {
            self.gateway = Some(gateway);
            Ok(self)
        }
    }

    pub fn set_nane(mut self, name: &str) -> Self {
        self.name = name.to_owned();
        self
    }

    pub fn done(self) -> Switch<T> {
        let router_addrs = self
            .endpoints
            .iter()
            .filter(|(_, val)| val.1)
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

        let switch = Switch {
            name: self.name,
            uuid: Uuid::new_v4(),
            ports,
            router_addrs,
            gateway: self.gateway,
        };

        trace!(
            "Switch(uuid={},name={}) is initialized: {:?}",
            &switch.uuid,
            &switch.name,
            &switch
        );
        switch
    }

    fn attach_endpoint(
        mut self,
        addr: Address,
        endpoint: Endpoint<T>,
        is_router: bool,
    ) -> Result<Self, SwitchError> {
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
    name: String,
    uuid: Uuid,
    ports: HashMap<Address, Port<T>>,
    router_addrs: Vec<Address>,
    gateway: Option<Address>,
}

impl<T: Clone + Debug> Switch<T> {
    pub fn builder() -> Builder<T> {
        Builder {
            endpoints: HashMap::new(),
            gateway: None,
            name: "".to_string(),
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
                trace!("[Switch({})] New data arrivas at port ({}): {:?}", self.uuid, ready_addr, pkt);
                // Process received packet
                pkt.set_saddr(ready_addr.clone());
                self.switch(&ready_addr, pkt);
            } else {
                trace!("[Switch({})] Port ({}) is closed", self.uuid, ready_addr);
                self.ports.remove(&ready_addr);
            }
        }
    }

    fn switch(&self, saddr: &Address, pkt: Packet<T>) {
        let ref_daddr = pkt.ref_daddr();

        match ref_daddr {
            Address::Broadcast => {
                self.broadcast(saddr, pkt);
            }
            Address::Addr(_) => {
                self.send_to(saddr, pkt);
            }
        }
    }

    fn broadcast(&self, saddr: &Address, pkt: Packet<T>) {
        trace!("[Switch({})] Braodcast pkt: {:?}", self.uuid, pkt);
        self.ports
            .iter()
            .filter(|(addr, _)| addr != &saddr)
            .map(|(_, port)| port)
            .for_each(|port| {
                port.send(pkt.clone());
            });
    }

    fn send_to(&self, _saddr: &Address, pkt: Packet<T>) {
        let ref_daddr = pkt.ref_daddr();

        let dst_port = self.ports.get(ref_daddr);

        if let Some(port) = dst_port {
            port.send(pkt);
        } else {
            // route or drop
            self.route_to(pkt);
        }
    }

    fn route_to(&self, pkt: Packet<T>) {
        let rt_info = pkt.ref_rt_info();

        let candidates = match rt_info {
            None => {
                // default gateway is used if packet is sent from local
                match self.gateway {
                    None => {
                        trace!("Packet is not sent to local, and not gateway is specificed, drop");
                        return;
                    }
                    Some(ref gateway) => {
                        vec![gateway]
                    }
                }
            }
            Some(rt_info) => self
                .router_addrs
                .iter()
                .filter(|&addr| *addr != rt_info.last_hop)
                .collect::<Vec<_>>(),
        };

        candidates.into_iter().for_each(|addr| {
            let port = self.ports.get(addr);
            match port {
                None => {
                    trace!("BUG: route addr is invalid");
                }
                Some(port) => {
                    if !port.is_router {
                        trace!("BUG: port is not router");
                    }
                    port.send(pkt.clone());
                }
            }
        });
    }
}

impl<T: Debug + Clone> Debug for Switch<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let router_addrs = self
            .router_addrs
            .iter()
            .fold(String::new(), |msg, addr| format!("{}\t\t{:?}\n", msg, addr));
        let ports = self.ports.iter().fold(String::new(), |msg, (addr, port)| {
            format!(
                "{}\t\t[Addr: {:?}, IsRouter: {:?}, WireId: {}, RxPeerId: {}, TxPeerId: {}]\n",
                msg,
                addr,
                port.is_router,
                port.rx.wire_id(),
                port.rx.peer_id(),
                port.tx.peer_id()
            )
        });

        let msg = format!(
            "Switch [{uuid}] {{\n\
             name: {name} \n\
             gateway: {gateway:?} \n\
             router_addrs: \n\
                {router_addrs} \n\
             ports: \n\
                {ports} \n\
            }}",
            uuid = &self.uuid,
            name = &self.name,
            gateway = &self.gateway,
            router_addrs = router_addrs,
            ports = ports,
        );

        write!(f, "{}", msg)
    }
}
