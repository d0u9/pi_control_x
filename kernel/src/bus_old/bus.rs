use ::futures::future;
use ::std::collections::HashMap;
use ::std::fmt::Debug;
use ::std::future::Future;

use super::address::{BusAddress, Address};
use super::endpoint::Packet;
use super::endpoint::Endpoint;

#[derive(Debug)]
pub struct Bus<T> {
    name: String,
    endpoints: HashMap<Address, Endpoint<T>>,
    gateway: Option<Address>,
    routers: Vec<Address>,
}

impl<T: Clone + Debug> Bus<T> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            endpoints: HashMap::new(),
            gateway: None,
            routers: Vec::new(),
        }
    }

    pub fn create_endpoint(&mut self, addr: &Address) -> Endpoint<T> {
        let endpoint = Endpoint::new(addr);
        let addr = (*addr).clone();
        let _ = self.endpoints.insert(addr, endpoint.clone());
        endpoint
    }

    pub fn set_gateway(&mut self, addr: &Address) {
        if self.endpoints.get(addr).is_none() {
            return;
        }
        let addr = (*addr).clone();
        self.gateway = Some(addr);
    }

    pub fn attach_router(&mut self, addr: &Address) {
        let addr = (*addr).clone();
        self.routers.push(addr);
    }

    pub async fn serve(self, shutdown: impl Future<Output = ()>) {
        tokio::select! {
            _ = self.poll() => {}
            _ = shutdown => {}
        }
    }

    async fn poll(&self) {
        let mut rx_pins = self
            .endpoints
            .values()
            .map(|x| x.pin_tx.subscribe())
            .collect::<Vec<_>>();

        loop {
            let pin_futures = rx_pins.iter_mut().map(|x| Box::pin(x.recv()));
            match future::select_all(pin_futures).await {
                (Ok(pkt), _, _) => {
                    self.process_pkt(pkt);
                }
                (Err(
                _ => {}
            }
        }
    }

    fn process_pkt(&self, pkt: Packet<T>) {
        println!(
            "BUS({:?}) [{:?} -> {:?}]: {:?}",
            self.name, pkt.src, pkt.dst, pkt.data
        );

        let dst_addr = &pkt.dst.clone();

        match dst_addr {
            BusAddress::Broadcast => {
                self.broadcast_packet_local(pkt);
            }
            BusAddress::Addr(ref addr) => {
                self.send_to(pkt, addr);
            }
            invalid_addr => {
                println!("Warn: Invalid dst addr type: {:?}", invalid_addr);
            }
        }
    }

    fn send_to(&self, pkt: Packet<T>, addr: &Address) {
        match self.get_endpoint_by_addr(addr) {
            Some(peer) => {
                peer.bus_send(pkt);
            }
            None => {
                self.send_to_no_local(pkt);
            }
        }
    }

    fn send_to_no_local(&self, pkt: Packet<T>) {
        let pkt_src = pkt.src.clone();

        match pkt_src {
            // Sent from bus internal
            BusAddress::Addr(_) => {
                if let Some(gateway) = &self.gateway {
                    self.send_to(pkt, gateway);
                }
            }
            // Sent from router
            BusAddress::Router(ref rt_addr) => {
                self.broadcast_to_routers(pkt, rt_addr.rt_addr());
            }
            _ => {}
        }
    }

    fn broadcast_to_routers(&self, pkt: Packet<T>, last: &Address) {
        let eps = self
            .endpoints
            .iter()
            .filter(|(addr, _)| *addr != last)
            .map(|(_, peer)| peer)
            .collect::<Vec<_>>();

        self.broadcast_packet(pkt, eps);
    }

    fn broadcast_packet(&self, pkt: Packet<T>, endpoints: Vec<&Endpoint<T>>) {
        endpoints.into_iter().for_each(|peer| {
            peer.bus_send(pkt.clone());
        })
    }

    fn broadcast_packet_local(&self, pkt: Packet<T>) {
        let src_addr = match pkt.src {
            BusAddress::Addr(ref addr) => addr,
            _ => return,
        };

        let iter = self
            .endpoints
            .iter()
            .filter(|(addr, _)| *addr != src_addr)
            .map(|(_, peer)| peer)
            .collect::<Vec<_>>();

        self.broadcast_packet(pkt, iter);
    }

    fn get_endpoint_by_addr(&self, addr: &Address) -> Option<&Endpoint<T>> {
        self.endpoints.get(addr)
    }
}
