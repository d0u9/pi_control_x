use ::std::fmt::Debug;
use ::tokio::sync::mpsc;

use super::*;

#[derive(Debug, Clone)]
pub enum RouterMode {
    FLAT,
    GATEWAY,
}

#[derive(Debug)]
pub struct Router<T, P> {
    mode: RouterMode,
    endpoints: Option<(Endpoint<T>, Endpoint<P>)>,
}

impl<T, P> Router<T, P>
where
    T: Debug + Clone + From<P>,
    P: Debug + Clone + From<T>,
{
    fn from_builder(builder: RouterBuilder) -> Self {
        let mode = builder.mode.expect("No mode is specified");
        Self {
            mode,
            endpoints: None,
        }
    }

    pub fn join(&mut self, ep0: Endpoint<T>, ep1: Endpoint<P>) {
        self.endpoints = Some((ep0, ep1))
    }

    pub fn mode(&self) -> RouterMode {
        self.mode.clone()
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

#[derive(Debug, Default)]
pub struct RouterBuilder {
    mode: Option<RouterMode>,
}

impl RouterBuilder {
    pub fn new() -> RouterBuilder {
        RouterBuilder::default()
    }

    pub fn mode(mut self, mode: RouterMode) -> Self {
        self.mode = Some(mode);
        self
    }

    pub fn create<T, P>(self) -> Router<T, P>
    where
        T: Clone + Debug + From<P>,
        P: Clone + Debug + From<T>,
    {
        Router::<T, P>::from_builder(self)
    }
}
