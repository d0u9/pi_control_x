use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::fmt::Debug;
use std::future::Future;

use log::{info, trace, warn};
use uuid::Uuid;

use super::address::Address;
use super::packet::{Packet, RouteInfo};
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

    pub fn set_name(mut self, name: &str) -> Self {
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
                    simplex: false,
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
    Ok(Packet<T>),
    Simplex,
}

#[derive(Debug)]
struct Port<T> {
    is_router: bool,
    addr: Address,
    rx: Rx<T>,
    tx: Tx<T>,
    simplex: bool,
}

impl<T: Clone + Debug> Port<T> {
    async fn poll(&mut self) -> (Address, PollResult<T>) {
        let result = match self.rx.recv().await {
            Ok(pkt) => {
                trace!("[Port({:?})] Receives new packet: {:?}", self.addr, pkt);
                PollResult::Ok(pkt)
            }
            Err(_) => PollResult::Simplex,
        };

        (self.get_addr(), result)
    }

    fn get_addr(&self) -> Address {
        self.addr.clone()
    }

    fn send(&self, pkt: Packet<T>) {
        trace!("[Port({:?})] Sent packet: {:?}", self.addr, pkt);
        self.tx.send_pkt(pkt);
    }

    fn set_to_simplex(&mut self) {
        self.simplex = true;
    }
}

pub struct Switch<T> {
    name: String,
    uuid: Uuid,
    ports: HashMap<Address, Port<T>>,
    router_addrs: HashSet<Address>,
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
        let uuid = self.uuid;
        tokio::select! {
            _ = shutdown => {
                info!("[Switch({})] Switch receives shutdown signal", uuid);
            },
            _ = self.inner_poll() => { },
        }
    }

    async fn inner_poll(mut self) {
        loop {
            let pin_futures = self
                .ports
                .iter_mut()
                .filter(|(_, port)| !port.simplex)
                .map(|(_, port)| Box::pin(port.poll()));

            let ((ready_addr, poll_result), _, _) = futures::future::select_all(pin_futures).await;

            if let PollResult::Ok(mut pkt) = poll_result {
                trace!(
                    "[Switch({})] New data arrivas at port ({}): {:?}",
                    self.uuid,
                    ready_addr,
                    pkt
                );
                // Process received packet
                self.tag_rt_info(&mut pkt, &ready_addr);
                self.switch(&ready_addr, pkt);
            } else {
                trace!("[Switch({})] Port ({}) is closed", self.uuid, ready_addr);
                if let Some(port) = self.ports.get_mut(&ready_addr) {
                    port.set_to_simplex();
                } else {
                    warn!("[BUG::Swich] Cannot find a polled port!");
                }
            }
        }
    }

    fn tag_rt_info(&self, pkt: &mut Packet<T>, ready_addr: &Address) {
        match self.router_addrs.get(ready_addr) {
            None => {
                // Received from normal endpoint
                pkt.set_saddr(ready_addr.to_owned());
            }
            Some(_) => {
                // Received from a router
                match pkt.ref_mut_rt_info() {
                    Some(rt_info) => {
                        rt_info.last_hop = ready_addr.to_owned();
                    }
                    None => {
                        pkt.set_rt_info(RouteInfo {
                            last_hop: ready_addr.to_owned(),
                        });
                    }
                }
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
            trace!(
                "[Switch({})] Packet is sent to local port: {:?}",
                self.uuid,
                port.addr
            );
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
                        trace!("[Switch({})] Not a local packet, and not gateway is specificed, drop!!", self.uuid);
                        return;
                    }
                    Some(ref gateway) => {
                        trace!(
                            "[Switch({})] Not a local packet, sent to gateway({})",
                            self.uuid,
                            gateway
                        );
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
                    warn!(
                        "[BUG::Swich] Addr {:?} should be bound to a port, but not",
                        addr
                    );
                }
                Some(port) => {
                    if !port.is_router {
                        warn!("[BUG::Swich] Addr {:?} should be a router, but not", addr);
                    }
                    port.send(pkt.clone());
                }
            }
        });
    }
}

impl<T: Debug + Clone> Debug for Switch<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let router_addrs = self.router_addrs.iter().fold(String::new(), |msg, addr| {
            format!("{}\t\t{:?}\n", msg, addr)
        });
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
