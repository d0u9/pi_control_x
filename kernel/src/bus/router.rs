use ::std::fmt::Debug;
use ::tokio::sync::mpsc;
use ::std::marker::PhantomData;

use super::*;

#[derive(Debug, Clone)]
pub enum RouterMode {
    FLAT,
    GATEWAY,
}

#[derive(Debug)]
pub struct Router<T, P> {
    mode: RouterMode,
    allow_broadcast: bool,
    endpoints: Option<(Endpoint<T>, Endpoint<P>)>,
}

impl<T, P> Router<T, P>
where
    T: Debug + Clone + From<P>,
    P: Debug + Clone + From<T>,
{
    pub fn build() -> Builder<T, P> {
        Builder::new()
    }
    pub fn join(&mut self, ep0: Endpoint<T>, ep1: Endpoint<P>) {
        self.endpoints = Some((ep0, ep1))
    }

    pub fn mode(&self) -> RouterMode {
        self.mode.clone()
    }

    pub fn allow_broadcast(&self) -> bool {
        self.allow_broadcast
    }

    pub async fn poll(self, mut shutdown: mpsc::Receiver<()>) {
        let (mut ep0, mut ep1) = self.endpoints.unwrap();
        loop {
            tokio::select! {
                src_pkt = ep0.recv_pkt() => {
                    let dst_pkt = Packet {
                        src: src_pkt.src,
                        dst: src_pkt.dst,
                        data: P::from(src_pkt.data),
                    };
                    ep1.send_pkt(dst_pkt);
                }
                src_pkt = ep1.recv_pkt() => {
                    let dst_pkt = Packet {
                        src: src_pkt.src,
                        dst: src_pkt.dst,
                        data: T::from(src_pkt.data),
                    };
                    ep0.send_pkt(dst_pkt);
                }
                _ = shutdown.recv() => {
                    break;
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct Builder<T, P> {
    mode: Option<RouterMode>,
    allow_broadcast: bool,
    _phantom: PhantomData<(T, P)>,
}

impl<T, P> Builder<T, P> 
where
    T: Debug + Clone + From<P>,
    P: Debug + Clone + From<T>,
{
    pub fn new() -> Builder<T, P> {
        Self {
            mode: None,
            allow_broadcast: false,
            _phantom: PhantomData,
        }

    }

    pub fn allow_broadcast(mut self) -> Self {
        self.allow_broadcast = true;
        self
    }

    pub fn mode(mut self, mode: RouterMode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn create(self) -> Router<T, P>
    where
        T: Clone + Debug + From<P>,
        P: Clone + Debug + From<T>,
    {
        let mode = self.mode.expect("No mode is specified");
        Router {
            mode,
            allow_broadcast: false,
            endpoints: None,
        }
    }
}
