use std::collections::{HashMap, HashSet};
use std::default::Default;
use std::fmt::Debug;
use std::future::Future;

use log::{info, trace, warn};

use super::address::Address;
use super::packet::{Packet, RouteInfo};
use super::wire::{Wire, Endpoint, Rx, Tx};
use super::types::DevId;

#[derive(Debug, Clone)]
pub enum SwitchError {
    AddressInvalid,
    AddressInUsed,
    UnknowCtrlErr,
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

        let control = SwitchCtrlEndpoint::new();

        let switch = Switch {
            name: self.name,
            id: DevId::new(),
            ports,
            router_addrs,
            mode: self.mode,
            control_endpoint: control,
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

    fn receives_cout(&self) -> usize {
        self.tx.receiver_count()
    }

    fn set_to_simplex(&mut self) {
        self.simplex = true;
    }
}

#[derive(Debug, Clone)]
pub enum ControlMsgRequest {
    CreateEndpoint(Address),
}

#[derive(Debug, Clone)]
pub enum ControlMsgResponse<T> {
    CreateEndpoint(Endpoint<T>),
    UNSPEC,
}

#[derive(Debug, Clone)]
pub enum ControlMsgErr {
    SwitchErr(SwitchError)
}

#[derive(Debug, Clone)]
pub enum ControlMsg<T> {
    Request(ControlMsgRequest),
    Response(ControlMsgResponse<T>),
    Err(ControlMsgErr),
}

pub struct Switch<T> {
    id: DevId,
    name: String,
    ports: HashMap<Address, Port<T>>,
    router_addrs: HashSet<Address>,
    mode: SwitchMode,
    control_endpoint: SwitchCtrlEndpoint<T>,
}

impl<T: Clone + Debug> Switch<T> {
    pub fn builder() -> Builder<T> {
        Builder {
            endpoints: HashMap::new(),
            mode: SwitchMode::Local,
            name: "".to_string(),
        }
    }

    pub fn human_id(&self) -> String {
        if self.name.is_empty() {
            self.id.to_string()
        } else {
            self.name.to_string()
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

    pub fn get_control_endpoint(&mut self) -> SwitchCtrl<T> {
        self.control_endpoint.get_peer()
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
        let human_id = self.human_id();
        trace!("[Switch({})] Start polling...", human_id);
        let (ctl_tx, mut ctl_rx) = self.control_endpoint.clone().split();

        enum PollDone<T> {
            Data((Address, PollResult<T>)),
            Control(ControlMsgRequest),
        }

        loop {
            let done = if self.ports.is_empty() {
                tokio::select! {
                    ctl_msg = ctl_rx.recv_request() => PollDone::Control(ctl_msg),
                }
            } else {
                let pin_futures = self
                    .ports
                    .iter_mut()
                    .filter(|(_, port)| !port.simplex)
                    .map(|(_, port)| Box::pin(port.poll()));

                let done = tokio::select! {
                    ((ready_addr, poll_result), _, _) = futures::future::select_all(pin_futures) => {
                        PollDone::Data((ready_addr, poll_result))
                    }
                    ctl_msg = ctl_rx.recv_request() => PollDone::Control(ctl_msg),
                };
                done
            };

            match done {
                PollDone::Data((ready_addr, poll_result)) => self.process_port_data(ready_addr, poll_result),
                PollDone::Control(ctl_msg) => self.process_control_data(ctl_msg, &ctl_tx),
            }
        }
    }

    fn process_control_data(&mut self, ctl_request: ControlMsgRequest, tx: &SwitchCtrlTx<T>) {
        match ctl_request {
            ControlMsgRequest::CreateEndpoint(addr) => {
                let (ep0, ep1) = Wire::endpoints::<T>();
                match self.attach(addr, ep0) {
                    Ok(_) => tx.send_response(ControlMsgResponse::CreateEndpoint(ep1)),
                    Err(e) => tx.send_err(ControlMsgErr::SwitchErr(e)),
                }
            }
        }
    }

    fn process_port_data(&mut self, ready_addr: Address, poll_result: PollResult<T>) {
        if let PollResult::Ok(mut pkt) = poll_result {
            trace!(
                "[Switch({})] New data arrivas at port ({}): {:?}",
                self.human_id(),
                ready_addr,
                pkt
                );
            // Process received packet
            self.tag_rt_info(&mut pkt, &ready_addr);
            self.switch(&ready_addr, pkt);
        } else {
            trace!("[Switch({})] Port ({}) is closed", self.human_id(), ready_addr);
            if let Some(port) = self.ports.get_mut(&ready_addr) {
                if port.receives_cout() > 0 {
                    port.set_to_simplex();
                } else {
                    self.ports.remove(&ready_addr);
                }
            } else {
                warn!("[BUG::Swich] Cannot find a polled port!");
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
            _ => {
            }
        }
    }

    fn broadcast(&self, saddr: &Address, pkt: Packet<T>) {
        trace!("[Switch({})] Braodcast pkt: {:?}", self.human_id(), pkt);
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
                self.human_id(),
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
                            self.human_id(),
                            gateway
                        );
                        vec![gateway]
                    }
                    SwitchMode::Broadcast => {
                        self.router_addrs.iter().collect::<Vec<_>>()
                    }
                    SwitchMode::Local => {
                        trace!("[Switch({})] Not a local packet, and not gateway is specificed, drop!!", self.human_id());
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

        if candidates.is_empty() {
            trace!("Packet is not receivers, DROP!!!!: {:?}", pkt);
            return;
        }

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
            id = &self.human_id(),
            name = &self.name,
            mode = &self.mode,
            router_addrs = router_addrs,
            ports = ports,
        );

        write!(f, "{}", msg)
    }
}

#[derive(Debug)]
pub struct SwitchCtrlTx<T> {
    inner: Tx<ControlMsg<T>>
}

impl<T> SwitchCtrlTx<T>
where
T: Clone + Debug
{
    pub fn send_request(&self, msg: ControlMsgRequest) {
        self.inner.send(Address::P2P, ControlMsg::Request(msg));
    }

    fn send_response(&self, msg: ControlMsgResponse<T>) {
        self.inner.send(Address::P2P, ControlMsg::Response(msg));
    }

    fn send_err(&self, msg: ControlMsgErr) {
        self.inner.send(Address::P2P, ControlMsg::Err(msg));
    }
}

#[derive(Debug)]
pub struct SwitchCtrlRx<T> {
    inner: Rx<ControlMsg<T>>
}

impl<T> SwitchCtrlRx<T>
where
T: Clone + Debug
{
    async fn recv_request(&mut self) -> ControlMsgRequest {
        loop {
            if let Ok(ControlMsg::Request(rqst)) = self.inner.recv_data().await {
                return rqst;
            }
        }
    }

    pub async fn recv_response(&mut self) -> Result<ControlMsgResponse<T>, ControlMsgErr> {
        loop {
            match self.inner.recv_data().await {
                Ok(ControlMsg::Response(rsps)) => { return Ok(rsps) }
                Ok(ControlMsg::Err(err)) => { return Err(err) }
                _ => { }
            }
        }
    }
}

#[derive(Clone, Debug)]
struct SwitchCtrlEndpoint<T> {
    inner: Endpoint<ControlMsg<T>>,
}

impl<T> SwitchCtrlEndpoint<T>
where
T: Clone + Debug
{
    fn new() -> Self {
        Self::default()
    }

    fn get_peer(&self) -> SwitchCtrl<T> {
        let peer = self.inner.get_peer();
        let (tx, rx) = peer.clone().split();
        SwitchCtrl {
            inner: peer,
            tx: SwitchCtrlTx{inner: tx},
            rx: SwitchCtrlRx{inner: rx},
        }
    }

    fn split(self) -> (SwitchCtrlTx<T>, SwitchCtrlRx<T>) {
        let (tx, rx) = self.inner.split();
        (SwitchCtrlTx{ inner: tx }, SwitchCtrlRx{ inner: rx })
    }

}

impl<T> Default for SwitchCtrlEndpoint<T>
where
T: Clone + Debug
{
    fn default() -> Self {
        let (control, _) = Wire::endpoints();
        Self{ inner: control }
    }
}

#[derive(Debug)]
pub struct SwitchCtrl<T> {
    inner: Endpoint<ControlMsg<T>>,
    tx: SwitchCtrlTx<T>,
    rx: SwitchCtrlRx<T>,
}

impl<T> Clone for SwitchCtrl<T>
where
    T: Debug + Clone
{
    fn clone(&self) -> Self {
        let inner = self.inner.clone();
        let (tx, rx) = inner.clone().split();
        SwitchCtrl {
            inner,
            tx: SwitchCtrlTx{inner: tx},
            rx: SwitchCtrlRx{inner: rx},
        }
    }
}

impl<T> SwitchCtrl<T>
where
    T: Debug + Clone
{
    pub async fn add_endpoint(&mut self, addr: Address) -> Result<Endpoint<T>, SwitchError> {
        self.tx.send_request(ControlMsgRequest::CreateEndpoint(addr));
        match self.rx.recv_response().await {
            Ok(ControlMsgResponse::CreateEndpoint(ep)) => Ok(ep),
            Err(ControlMsgErr::SwitchErr(e)) => Err(e),
            _ => Err(SwitchError::UnknowCtrlErr),
        }
    }
}

