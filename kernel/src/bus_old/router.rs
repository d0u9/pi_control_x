use ::std::fmt::Debug;
use ::std::marker::PhantomData;
use ::tokio::sync::mpsc;

use super::policy::Policy;
use super::endpoint::Endpoint;
use super::address::{BusAddress, RouterAddr};

#[derive(Debug, Clone)]
pub enum RouterMode {
    FLAT,
    // GATEWAY,
}

#[derive(Debug)]
pub struct Router<S, D> {
    policy: Policy,
    endpoints: Option<(Endpoint<S>, Endpoint<D>)>,
}

impl<S, D> Router<S, D>
where
    S: Debug + Clone + From<D>,
    D: Debug + Clone + From<S>,
{
    pub fn build() -> Builder<S, D> {
        Builder::new()
    }
    pub fn join(&mut self, ep0: Endpoint<S>, ep1: Endpoint<D>) {
        self.endpoints = Some((ep0, ep1))
    }

    pub fn mode(&self) -> RouterMode {
        self.policy.mode.clone()
    }

    pub fn allow_broadcast(&self) -> bool {
        self.policy.allow_broadcast
    }

    pub async fn poll(self, mut shutdown: mpsc::Receiver<()>) {
        let (mut ep0, mut ep1) = self.endpoints.unwrap();
        let policy = &self.policy;
        loop {
            tokio::select! {
                src_pkt = ep0.recv_pkt() => {
                    if let Some(pkt) = policy.route_packet(src_pkt) {
                        ep1.send_pkt(pkt);
                    }
                }
                src_pkt = ep1.recv_pkt() => {
                    if let Some(pkt) = policy.route_packet(src_pkt) {
                        ep0.send_pkt(pkt);
                    }
                }
                _ = shutdown.recv() => {
                    break;
                }
            }
        }
    }

    // TODO: 
    fn update_src_addr(src: &BusAddress, send_endpoint: &BusAddress) -> Option<BusAddress> {
        let send_endpoint_addr = match send_endpoint {
            BusAddress::Broadcast | BusAddress::Router(_) => {
                return None;
            }
            BusAddress::Addr(addr) => { addr }
        };


        match src {
            BusAddress::Broadcast => {
                println!("Src address is Broadcast, drop!");
                None
            }
            BusAddress::Addr(addr) => {
                Some(BusAddress::Router(RouterAddr::new(send_endpoint_addr, addr)))
            },
            BusAddress::Router(addr) => {
                let mut addr = addr.clone();
                addr.set_last_router(send_endpoint_addr);
                Some(BusAddress::Router(addr.clone()))
            },
        }
    }
}

#[derive(Debug)]
pub struct Builder<S, D> {
    mode: Option<RouterMode>,
    allow_broadcast: bool,
    _phantom: PhantomData<(S, D)>,
}

impl<S, D> Builder<S, D>
where
    S: Debug + Clone + From<D>,
    D: Debug + Clone + From<S>,
{
    pub fn new() -> Builder<S, D> {
        Self {
            mode: None,
            allow_broadcast: false,
            _phantom: PhantomData,
        }
    }

    pub fn allow_broadcast(mut self) -> Self {
        self.allow_broadcast = true;
        self
    }

    pub fn mode(mut self, mode: RouterMode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn create(self) -> Router<S, D>
    where
        S: Clone + Debug + From<D>,
        D: Clone + Debug + From<S>,
    {
        let mode = self.mode.expect("No mode is specified");
        Router {
            policy: Policy {
                mode,
                allow_broadcast: false,
            },
            endpoints: None,
        }
    }
}
