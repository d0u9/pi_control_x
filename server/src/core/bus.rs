use super::EventEnum;
use ::tokio::sync::broadcast;

pub type BusSender = broadcast::Sender<EventEnum>;
pub type BusReceiver = broadcast::Receiver<EventEnum>;

#[derive(Clone, Debug)]
pub struct Bus(broadcast::Sender<EventEnum>);

impl Bus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(32);
        Bus(tx)
    }

    pub fn sender(&self) -> BusSender {
        self.0.clone()
    }

    pub fn receiver(&self) -> BusReceiver {
        self.0.subscribe()
    }
}
