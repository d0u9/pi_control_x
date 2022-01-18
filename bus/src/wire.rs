#![cfg_attr(test, allow(dead_code))]
use std::fmt::Debug;

use log::trace;
use tokio::time::{self, Duration};
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;

use super::address::Address;
use super::packet::Packet;
use super::types::DevId;

type RawRx<T> = broadcast::Receiver<T>;
type RawTx<T> = broadcast::Sender<T>;

#[derive(Debug)]
pub enum EndpointError {
    AddressError,
    Timeout,
    Closed,
}

#[derive(Debug)]
pub struct Rx<T> {
    wire: DevId,
    peer: DevId,
    rx: RawRx<Packet<T>>,
}

impl<T: Debug + Clone> Rx<T> {
    pub async fn recv_data_timeout(&mut self, timeout: Duration) -> Result<T, EndpointError> {
        self.recv_timeout(timeout).await.map(|pkt| pkt.into_val())
    }

    pub async fn recv_data(&mut self) -> Result<T, EndpointError> {
        self.recv().await.map(|pkt| pkt.into_val())
    }

    pub async fn recv_data_addr_timeout(&mut self, timeout: Duration) -> Result<(T, Address, Address), EndpointError> {
        let pkt = self.recv_timeout(timeout).await?;

        let saddr = pkt.get_saddr().ok_or(EndpointError::AddressError)?;
        let daddr = pkt.get_daddr();
        let val = pkt.into_val();
        Ok((val, saddr, daddr))
    }

    pub async fn recv_data_addr(&mut self) -> Result<(T, Address, Address), EndpointError> {
        let pkt = self.recv().await?;

        let saddr = pkt.get_saddr().ok_or(EndpointError::AddressError)?;
        let daddr = pkt.get_daddr();
        let val = pkt.into_val();
        Ok((val, saddr, daddr))
    }

    pub(super) async fn recv_timeout(&mut self, timeout: Duration) -> Result<Packet<T>, EndpointError> {
        let wait = time::sleep(timeout);
        tokio::select! {
            data = self.recv() => {
                data
            }
            _ = wait => {
                Err(EndpointError::Timeout)
            }
        }
    }

    pub(super) async fn recv(&mut self) -> Result<Packet<T>, EndpointError> {
        loop {
            match self.rx.recv().await {
                Ok(val) => {
                    trace!("[Rx({})] Recieves new packet: {:?}", self.peer, val);
                    return Ok(val);
                }
                Err(RecvError::Closed) => {
                    trace!("[Rx({})] Is closed", self.peer);
                    return Err(EndpointError::Closed);
                }
                Err(RecvError::Lagged(num)) => {
                    trace!("[Rx({})] Has lagged {} packets, retry", self.peer, num);
                    continue;
                }
            }
        }
    }

    pub fn wire_id(&self) -> DevId {
        self.wire
    }

    pub fn peer_id(&self) -> DevId {
        self.peer
    }
}

#[derive(Debug)]
pub struct Tx<T> {
    wire: DevId,
    peer: DevId,
    tx: RawTx<Packet<T>>,
}

impl<T: Debug + Clone> Tx<T> {
    pub fn send(&self, daddr: Address, val: T) {
        let pkt = Packet::new(daddr, val);
        self.send_pkt(pkt)
    }

    pub fn send_data(&self, val: T) {
        let pkt = Packet::new(Address::Broadcast, val);
        self.send_pkt(pkt)
    }

    pub fn send_pkt(&self, pkt: Packet<T>) {
        trace!("[Tx({})] Send packet: {:?}", self.peer, pkt);

        if let Err(e) = self.tx.send(pkt) {
            trace!("Send Packet failed: packet dropped: {:?}", e.0);
        }
    }

    pub fn wire_id(&self) -> DevId {
        self.wire
    }

    pub fn peer_id(&self) -> DevId {
        self.peer
    }

    pub fn receiver_count(&self) -> usize {
        self.tx.receiver_count()
    }
}

#[derive(Debug, Clone)]
pub struct Endpoint<T> {
    peer: DevId,
    wire: DevId,
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

    pub fn get_peer(&self) -> Self {
        Self {
            peer: self.peer,
            wire: self.wire,
            tx_this: self.tx_that.clone(),
            tx_that: self.tx_this.clone(),
        }
    }
}

pub struct Wire;

impl Wire {
    pub fn endpoints<T: Debug + Clone>() -> (Endpoint<T>, Endpoint<T>) {
        let (tx0, _) = broadcast::channel(16);
        let (tx1, _) = broadcast::channel(16);
        let wire0 = DevId::new();
        let wire1 = wire0;

        let ep0 = Endpoint {
            peer: DevId::new(),
            wire: wire0,
            tx_this: tx0.clone(),
            tx_that: tx1.clone(),
        };

        let ep1 = Endpoint {
            peer: DevId::new(),
            wire: wire1,
            tx_this: tx1,
            tx_that: tx0,
        };

        (ep0, ep1)
    }
}
