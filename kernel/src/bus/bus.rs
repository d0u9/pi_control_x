use ::futures::future;
use ::std::collections::HashMap;
use ::std::fmt::Debug;
use ::std::future::Future;
use ::tokio::sync::broadcast;
use ::tokio::sync::mpsc;

use super::router::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Address {
    addr: String,
}

impl Address {
    pub fn new(addr: &str) -> Address {
        Self {
            addr: addr.to_owned(),
        }
    }
}

#[derive(Clone, Hash, PartialEq, Eq)]
enum BusAddress {
    Broadcast,
    Addr(Address),
}

#[derive(Clone, Debug)]
struct Packet<T> {
    src: Address,
    dst: Address,
    data: T,
}

pub struct Endpoint<T> {
    addr: Address,
    pin_tx: broadcast::Sender<Packet<T>>,
    pin_rx: broadcast::Receiver<Packet<T>>,
    bus_tx: broadcast::Sender<Packet<T>>,
    bus_rx: broadcast::Receiver<Packet<T>>,
}

impl<T: Clone> Clone for Endpoint<T> {
    fn clone(&self) -> Self {
        Self {
            addr: self.addr.clone(),
            pin_tx: self.pin_tx.clone(),
            pin_rx: self.bus_tx.subscribe(),
            bus_tx: self.bus_tx.clone(),
            bus_rx: self.pin_tx.subscribe(),
        }
    }
}

impl<T: Clone + Debug> Endpoint<T> {
    fn new(addr: &Address) -> Endpoint<T> {
        let (bus_tx, pin_rx) = broadcast::channel(16);
        let (pin_tx, bus_rx) = broadcast::channel(16);

        Self {
            addr: (*addr).clone(),
            pin_tx,
            pin_rx,
            bus_tx,
            bus_rx,
        }
    }

    pub fn send(&self, dst: &Address, data: T) {
        self.pin_tx
            .send(Packet {
                src: self.addr.clone(),
                dst: (*dst).clone(),
                data,
            })
            .unwrap();
    }

    pub async fn recv(&mut self) -> (Address, T) {
        let packet = self.pin_rx.recv().await.unwrap();
        (packet.src, packet.data)
    }

    fn bus_send(&self, packet: Packet<T>) {
        self.bus_tx.send(packet).unwrap();
    }
}

pub struct Bus<T> {
    name: String,
    endpoints: HashMap<Address, Endpoint<T>>,
}

impl<T: Clone + Debug> Bus<T> {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            endpoints: HashMap::new(),
        }
    }

    pub fn crate_endpoint(&mut self, addr: &Address) -> Endpoint<T> {
        let endpoint = Endpoint::new(addr);
        let addr = (*addr).clone();
        let _ = self.endpoints.insert(addr, endpoint.clone());
        endpoint
    }

    pub async fn poll(self, mut shutdown: mpsc::Receiver<()>) {
        let endpoints = self.endpoints;
        let mut bus_rxs = endpoints
            .values()
            .map(|x| x.pin_tx.subscribe())
            .collect::<Vec<_>>();
        loop {
            let pin_futures_iter = bus_rxs.iter_mut().map(|x| Box::pin(x.recv()));
            tokio::select! {
                (Ok(packet), _, _) = future::select_all(pin_futures_iter) => {
                    println!("BUS({:?}) [{:?} -> {:?}]: {:?}", self.name, packet.src, packet.dst, packet.data);
                    let peer = endpoints.get(&packet.dst).unwrap();
                    peer.bus_send(packet);
                }
                _ = shutdown.recv() => {
                    break;
                }
            }
        }
    }
}
