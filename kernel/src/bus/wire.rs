#![cfg_attr(test, allow(dead_code))]
use std::fmt::Debug;

use log::trace;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

use super::address::Address;
use super::packet::Packet;

type RawRx<T> = broadcast::Receiver<T>;
type RawTx<T> = broadcast::Sender<T>;

#[derive(Debug)]
pub enum EndpointError {
    Closed,
}

#[derive(Debug)]
pub struct Rx<T> {
    wire: Uuid,
    peer: Uuid,
    rx: RawRx<Packet<T>>,
}

impl<T: Debug + Clone> Rx<T> {
    pub async fn recv(&mut self) -> Result<Packet<T>, EndpointError> {
        loop {
            match self.rx.recv().await {
                Ok(val) => return Ok(val),
                Err(RecvError::Closed) => return Err(EndpointError::Closed),
                Err(RecvError::Lagged(num)) => {
                    trace!("Endpoint {} has lagged {} packets, retry", self.peer, num);
                    continue;
                }
            }
        }
    }

    pub fn wire_id(&self) -> Uuid {
        self.wire
    }

    pub fn peer_id(&self) -> Uuid {
        self.peer
    }
}

#[derive(Debug)]
pub struct Tx<T> {
    wire: Uuid,
    peer: Uuid,
    tx: RawTx<Packet<T>>,
}

impl<T: Debug + Clone> Tx<T> {
    pub fn send(&self, daddr: Address, val: T) {
        let pkt = Packet::new(daddr, val);
        self.send_pkt(pkt)
    }

    pub fn send_pkt(&self, pkt: Packet<T>) {
        if let Err(e) = self.tx.send(pkt) {
            trace!("Send Packet failed: packet dropped: {:?}", e.0);
        }
    }

    pub fn wire_id(&self) -> Uuid {
        self.wire
    }

    pub fn peer_id(&self) -> Uuid {
        self.peer
    }
}

#[derive(Debug)]
pub struct Endpoint<T> {
    peer: Uuid,
    wire: Uuid,
    tx_this: RawTx<Packet<T>>,
    tx_that: RawTx<Packet<T>>,
}

impl<T: Clone + Debug> Endpoint<T> {
    pub fn split(self) -> (Tx<T>, Rx<T>) {
        let rx = Rx {
            wire: self.wire,
            peer: self.peer,
            rx: self.tx_that.subscribe(),
        };

        let tx = Tx {
            wire: self.wire,
            peer: self.peer,
            tx: self.tx_this,
        };

        (tx, rx)
    }
}

pub struct Wire;

impl Wire {
    pub fn endpoints<T: Debug + Clone>() -> (Endpoint<T>, Endpoint<T>) {
        let (tx0, _) = broadcast::channel(16);
        let (tx1, _) = broadcast::channel(16);
        let wire0 = Uuid::new_v4();
        let wire1 = wire0;

        let ep0 = Endpoint {
            peer: Uuid::new_v4(),
            wire: wire0,
            tx_this: tx0.clone(),
            tx_that: tx1.clone(),
        };

        let ep1 = Endpoint {
            peer: Uuid::new_v4(),
            wire: wire1,
            tx_this: tx1,
            tx_that: tx0,
        };

        (ep0, ep1)
    }
}
