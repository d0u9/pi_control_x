pub mod result;
pub mod udev;


pub(crate) mod Shutdown {
    pub struct ShutdownSender(::tokio::sync::oneshot::Sender<()>);
    pub struct ShutdownReceiver(::tokio::sync::oneshot::Receiver<()>);

    pub fn new() -> (ShutdownSender, ShutdownReceiver) {
        let (tx, rx) = ::tokio::sync::oneshot::channel();
        ( ShutdownSender(tx), ShutdownReceiver(rx) )
    }

    impl ShutdownSender {
        pub fn shutdown(self) {
            self.0.send(()).unwrap();
        }
    }

    impl ShutdownReceiver {
        pub async fn wait(&mut self) {
            let inner = &mut self.0;
            inner.await;
        }
    }
}
