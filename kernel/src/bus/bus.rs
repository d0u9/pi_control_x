use ::futures::future;
use ::std::collections::HashMap;
use ::std::fmt::Debug;
use ::tokio::sync::mpsc;

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

    pub async fn poll(self, mut shutdown: mpsc::Receiver<()>) {
        let mut bus_rxs = self
            .endpoints
            .values()
            .map(|x| x.pin_tx.subscribe())
            .collect::<Vec<_>>();
        loop {
            let pin_futures_iter = bus_rxs.iter_mut().map(|x| Box::pin(x.recv()));
            tokio::select! {
                (Ok(packet), _, _) = future::select_all(pin_futures_iter) => {
                    println!("BUS({:?}) [{:?} -> {:?}]: {:?}", self.name, packet.src, packet.dst, packet.data);
                    match packet.dst {
                        BusAddress::Addr(ref addr) => {
                            match self.endpoints.get(addr) {
                                Some(peer) => {
                                    peer.bus_send(packet);
                                }
                                None => match self.gateway {
                                    Some(ref gateway) => {
                                        let peer = self.endpoints.get(gateway).unwrap();
                                        peer.bus_send(packet)
                                    }
                                    None => {
                                        println!("Dropped!!!");
                                    }
                                }
                            }

                        }
                        BusAddress::Broadcast => {
                            self.broadcast_packet(packet);
                        }
                    }
                }
                _ = shutdown.recv() => {
                    break;
                }
            }
        }
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
