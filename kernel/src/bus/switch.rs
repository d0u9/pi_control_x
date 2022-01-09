use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::fmt::Debug;
use std::future::Future;

use log::{info, trace, warn};

use super::address::Address;
use super::packet::{Packet, RouteInfo};
use super::wire::{Endpoint, Rx, Tx};
use super::types::DevId;

#[derive(Debug)]
pub enum SwitchError {
    AddressInvalid,
    AddressInUsed,
}

#[derive(Debug)]
pub enum SwitchMode {
    // packets sent not to local switch are directed to the gateway
    Gateway(Address),
    // packets sent not to local are broadcasted to all router ports
    Broadcast,
    // packets sent not to local are droped.
    Local,
}

pub struct Builder<T> {
    // bool = true represents a router
    endpoints: HashMap<Address, (Endpoint<T>, bool)>,
    mode: SwitchMode,
    name: String,
}

impl<T: Debug + Clone> Builder<T> {
    pub fn attach(self, addr: Address, endpoint: Endpoint<T>) -> Result<Self, SwitchError> {
        self.attach_endpoint(addr, endpoint, false)
    }

    pub fn attach_router(self, addr: Address, endpoint: Endpoint<T>) -> Result<Self, SwitchError> {
        self.attach_endpoint(addr, endpoint, true)
    }

    pub fn set_mode(mut self, mode: SwitchMode) -> Result<Self, SwitchError> {
        let gateway = match mode {
            SwitchMode::Gateway(gateway) => gateway,
            _ => {
                self.mode = mode;
                return Ok(self)
            },
        };

        let (_, is_router) = self
            .endpoints
            .get(&gateway)
            .ok_or(SwitchError::AddressInvalid)?;
        if !(*is_router) {
            Err(SwitchError::AddressInvalid)
        } else {
            self.mode = SwitchMode::Gateway(gateway);
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
            id: DevId::new(),
            ports,
            router_addrs,
            mode: self.mode,
        };

        trace!(
            "Switch(id={},name={}) is initialized: {:?}",
            &switch.id,
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
    id: DevId,
    name: String,
    ports: HashMap<Address, Port<T>>,
    router_addrs: HashSet<Address>,
    mode: SwitchMode,
}

impl<T: Clone + Debug> Switch<T> {
    pub fn builder() -> Builder<T> {
        Builder {
            endpoints: HashMap::new(),
            mode: SwitchMode::Local,
            name: "".to_string(),
        }
    }

    pub async fn poll(self) {
        self.inner_poll().await
    }

    pub async fn poll_with_graceful(self, shutdown: impl Future<Output = ()>) {
        let id = self.id;
        tokio::select! {
            _ = shutdown => {
                info!("[Switch({})] Switch receives shutdown signal", id);
            },
            _ = self.inner_poll() => { },
        }
    }

    pub fn get_id(&self) -> DevId {
        self.id
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn attach_router(&mut self, addr: Address, endpoint: Endpoint<T>) -> Result<(), SwitchError> {
        self.router_addrs.insert(addr.clone());
        self.attach_endpoint(addr, endpoint, true)
    }

    pub fn attach(&mut self, addr: Address, endpoint: Endpoint<T>) -> Result<(), SwitchError> {
        self.attach_endpoint(addr, endpoint, false)
    }

    pub fn attach_endpoint(&mut self, addr: Address, endpoint: Endpoint<T>, is_router: bool) -> Result<(), SwitchError> {
        if self.ports.get(&addr).is_some() {
            return Err(SwitchError::AddressInUsed);
        }

        let (tx, rx) = endpoint.split();

        let port = Port {
            is_router,
            addr: addr.clone(),
            tx,
            rx,
            simplex: false
        };

        self.ports.insert(addr, port);

        Ok(())
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
                    self.id,
                    ready_addr,
                    pkt
                );
                // Process received packet
                self.tag_rt_info(&mut pkt, &ready_addr);
                self.switch(&ready_addr, pkt);
            } else {
                trace!("[Switch({})] Port ({}) is closed", self.id, ready_addr);
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
        trace!("[Switch({})] Braodcast pkt: {:?}", self.id, pkt);
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
                self.id,
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
                match self.mode {
                    SwitchMode::Gateway(ref gateway) => {
                        trace!(
                            "[Switch({})] Not a local packet, sent to gateway({})",
                            self.id,
                            gateway
                        );
                        vec![gateway]
                    }
                    SwitchMode::Broadcast => {
                        self.router_addrs.iter().collect::<Vec<_>>()
                    }
                    SwitchMode::Local => {
                        trace!("[Switch({})] Not a local packet, and not gateway is specificed, drop!!", self.id);
                        return;
                    }
                }
            }
            Some(rt_info) => self
                .router_addrs
                .iter()
                .filter(|&addr| *addr != rt_info.last_hop)
                .collect::<Vec<_>>(),
        };

        trace!("----------- {:?}", candidates);

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
            "Switch [{id}] {{\n\
             name: {name} \n\
             gateway: {mode:?} \n\
             router_addrs: \n\
                {router_addrs} \n\
             ports: \n\
                {ports} \n\
            }}",
            id = &self.id,
            name = &self.name,
            mode = &self.mode,
            router_addrs = router_addrs,
            ports = ports,
        );

        write!(f, "{}", msg)
    }
}
