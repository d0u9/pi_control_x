use ::std::fmt::Debug;
use ::tokio::sync::broadcast;

use super::address::{BusAddress, Address};

pub(super) type TxPin<T> = broadcast::Sender<Packet<T>>;
pub(super) type RxPin<T> = broadcast::Receiver<Packet<T>>;

#[derive(Clone, Debug)]
pub(super) struct Packet<T> {
    pub(super) src: BusAddress,
    pub(super) dst: BusAddress,
    pub(super) data: T,
}

#[derive(Debug)]
pub struct Endpoint<T> {
    pub(super) addr: BusAddress,
    pub(super) pin_tx: TxPin<T>,
    pub(super) pin_rx: RxPin<T>,
    pub(super) bus_tx: TxPin<T>,
    pub(super) bus_rx: RxPin<T>,
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
    fn with_bus_addr(addr: BusAddress) -> Endpoint<T> {
        let (bus_tx, pin_rx) = broadcast::channel(16);
        let (pin_tx, bus_rx) = broadcast::channel(16);

        Self {
            addr,
            pin_tx,
            pin_rx,
            bus_tx,
            bus_rx,
        }
    }

    pub(super) fn new(addr: &Address) -> Endpoint<T> {
        let addr = (*addr).clone();
        Self::with_bus_addr(BusAddress::Addr(addr))
    }

    pub fn send(&self, dst: &Address, data: T) {
        let addr = (*dst).clone();
        self.send_pkt(Packet {
            src: self.addr.clone(),
            dst: BusAddress::Addr(addr),
            data,
        })
    }

    pub fn broadcast(&self, data: T) {
        let packet = Packet {
            src: self.addr.clone(),
            dst: BusAddress::Broadcast,
            data,
        };

        self.send_pkt(packet)
    }

    pub(super) fn send_pkt(&self, packet: Packet<T>) {
        self.pin_tx.send(packet).unwrap();
    }

    pub async fn recv(&mut self) -> Option<(Address, T)> {
        let packet = self.pin_rx.recv().await.unwrap();
        if let BusAddress::Addr(addr) = packet.src {
            Some((addr, packet.data))
        } else {
            None
        }
    }

    pub(super) async fn recv_pkt(&mut self) -> Packet<T> {
        self.pin_rx.recv().await.unwrap()
    }

    pub(super) fn bus_send(&self, packet: Packet<T>) {
        self.bus_tx.send(packet).unwrap();
    }
}
