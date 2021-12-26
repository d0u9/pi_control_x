use ::std::fmt::Debug;
use ::std::collections::HashMap;
use ::std::future::Future;
use ::futures::future;
use ::tokio::sync::broadcast;
use ::tokio::sync::mpsc;

use super::router::*;

#[derive(Hash, PartialEq, Eq)]
pub struct Address {
    addr: String,
}

impl Address {
    pub fn new(addr: &str) -> Address {
        Self { addr: addr.to_owned() }
    }
}

#[derive(Hash, PartialEq, Eq)]
enum BusAddress {
    Broadcast,
    Addr(Address),
}

pub struct Endpoint<T> {
    tx: broadcast::Sender<T>,
    rx: broadcast::Receiver<T>,
}

impl<T: Clone + Debug> Endpoint<T> {
    fn new() -> (Endpoint<T>, Endpoint<T>) {
        let (bus_tx, pin_rx) = broadcast::channel(16);
        let (pin_tx, bus_rx) = broadcast::channel(16);
        let bus_endpoint = Self {
            tx: bus_tx,
            rx: bus_rx,
        };
        let pin_endpoint = Self {
            tx: pin_tx,
            rx: pin_rx,
        };
        (bus_endpoint, pin_endpoint)
    }

    pub fn send(&self, val: T) {
        self.tx.send(val).unwrap();
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

    pub fn crate_endpoint(&mut self, addr: Address) -> Endpoint<T> {
        let (bus_point, pin_point) = Endpoint::new();
        let _ = self.endpoints.insert(addr, bus_point);
        pin_point
    }

    pub async fn poll(self, mut shutdown: mpsc::Receiver<()>) {
        let mut endpoints = self.endpoints;
        loop {
            let futures_iter = endpoints.values_mut().map(|x| Box::pin(x.rx.recv()));
            tokio::select! {
                (output, _, _) = future::select_all(futures_iter) => {
                    dbg!(output);
                }
                _ = shutdown.recv() => {
                    break;
                }
            }
        }
    }
}
