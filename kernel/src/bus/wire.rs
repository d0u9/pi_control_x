use std::fmt::Debug;

use log::trace;
use tokio::sync::broadcast;
use tokio::sync::broadcast::error::RecvError;
use uuid::Uuid;

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
    rx: RawRx<T>,
}

impl<T: Debug + Clone> Rx<T> {
    pub async fn recv(&mut self) -> Result<T, EndpointError> {
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
}

#[derive(Debug)]
pub struct Tx<T> {
    wire: Uuid,
    peer: Uuid,
    tx: RawTx<T>,
}

impl<T: Debug + Clone> Tx<T> {
    pub fn send(&self, val: T) {
        self.tx.send(val).expect("send failed");
    }
}

#[derive(Debug)]
pub struct Endpoint<T> {
    peer: Uuid,
    wire: Uuid,
    tx_this: RawTx<T>,
    tx_that: RawTx<T>,
}

impl<T: Clone + Debug> Endpoint<T> {
    pub fn split(self) -> (Tx<T>, Rx<T>) {
        let rx = Rx {
            wire: self.wire.clone(),
            peer: self.peer.clone(),
            rx: self.tx_that.subscribe(),
        };

        let tx = Tx {
            wire: self.wire.clone(),
            peer: self.peer.clone(),
            tx: self.tx_this,
        };

        (tx, rx)
    }
}

pub struct Wire;

impl Wire {
    pub fn new<T: Debug + Clone>() -> (Endpoint<T>, Endpoint<T>) {
        let (tx0, _) = broadcast::channel(16);
        let (tx1, _) = broadcast::channel(16);
        let wire0 = Uuid::new_v4();
        let wire1 = wire0.clone();

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
