use ::futures::future;
use ::std::collections::HashMap;
use ::std::fmt::Debug;
use ::std::future::Future;

use super::*;

#[derive(Debug)]
pub struct Bus<T> {
    name: String,
    endpoints: HashMap<Address, Endpoint<T>>,
    gateway: Option<Address>,
}

impl<T: Clone + Debug> Bus<T> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            endpoints: HashMap::new(),
            gateway: None,
        }
    }

    pub fn create_endpoint(&mut self, addr: &Address) -> Endpoint<T> {
        let endpoint = Endpoint::new(addr);
        let addr = (*addr).clone();
        let _ = self.endpoints.insert(addr, endpoint.clone());
        endpoint
    }

    pub fn set_gateway(&mut self, addr: &Address) {
        // TODO: test if addr is valid
        let addr = (*addr).clone();
        self.gateway = Some(addr);
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
                self.broadcast_packet(pkt);
            }
            BusAddress::Addr(ref addr) => {
                self.send_to(pkt, addr);
            }
        }
    }

    fn send_to(&self, pkt: Packet<T>, addr: &Address) {
        match self.get_endpoint_by_addr(addr) {
            Some(peer) => {
                peer.bus_send(pkt);
            }
            None => {
                if let Some(gateway) = &self.gateway {
                    self.send_to(pkt, gateway);
                }
            }
        }
    }

    fn get_endpoint_by_addr(&self, addr: &Address) -> Option<&Endpoint<T>> {
        self.endpoints.get(addr)
    }

    fn broadcast_packet(&self, packet: Packet<T>) {
        let src_addr = match packet.src {
            BusAddress::Addr(ref addr) => addr,
            _ => return,
        };

        self.endpoints
            .iter()
            .filter(|(addr, _)| *addr != src_addr)
            .for_each(|(_, peer)| {
                let mut packet = packet.clone();
                packet.dst = peer.addr.clone();
                peer.bus_send(packet);
            });
    }
}
