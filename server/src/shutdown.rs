use ::tokio::sync::oneshot;

pub struct ShutdownSender(oneshot::Sender<()>);
pub struct ShutdownReceiver(oneshot::Receiver<()>);

pub fn new() -> (ShutdownSender, ShutdownReceiver) {
    let (tx, rx) = oneshot::channel();
    (ShutdownSender(tx), ShutdownReceiver(rx))
}

impl ShutdownSender {
    pub fn shutdown(self) {
        self.0.send(()).unwrap_or_else(|e| {
            println!("Shutdown send filed: {:?}", e);
        });
    }
}

impl ShutdownReceiver {
    pub async fn wait(&mut self) {
        let inner = &mut self.0;
        let _ = inner.await;
    }
}
