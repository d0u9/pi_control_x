use ::tokio::sync::broadcast;

pub type BusSender<T> = broadcast::Sender<T>;
pub type BusReceiver<T> = broadcast::Receiver<T>;

#[derive(Clone, Debug)]
pub struct Bus<T>(broadcast::Sender<T>);

impl<T: Clone> Bus<T> {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(32);
        Bus(tx)
    }

    pub fn sender(&self) -> BusSender<T> {
        self.0.clone()
    }

    pub fn receiver(&self) -> BusReceiver<T> {
        self.0.subscribe()
    }
}
