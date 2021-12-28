use ::std::fmt::Debug;

use super::*;

#[derive(Debug)]
pub(super) struct Policy {
    pub(super) mode: RouterMode,
    pub(super) allow_broadcast: bool,
}

impl Policy {
    pub(super) fn route_packet<S, D>(&self, src_pkt: Packet<S>) -> Option<Packet<D>>
    where
        S: Debug + Clone + From<D>,
        D: Debug + Clone + From<S>,
    {
        if BusAddress::Broadcast == src_pkt.dst && !self.allow_broadcast {
            return None;
        }

        let dst_pkt = Packet {
            src: src_pkt.src,
            dst: src_pkt.dst,
            data: D::from(src_pkt.data),
        };

        Some(dst_pkt)
    }
}
