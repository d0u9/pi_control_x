#![cfg_attr(test, allow(dead_code))]
use std::fmt::Debug;

use super::address::Address;

#[derive(Debug, Clone)]
pub struct Packet<T> {
    val: T,
    daddr: Address,
    saddr: Option<Address>,
}

impl<T: Clone + Debug> Packet<T> {
    pub fn new(daddr: Address, val: T) -> Self {
        Self {
            saddr: None,
            daddr,
            val,
        }
    }

    pub fn set_saddr(&mut self, saddr: Address) {
        self.saddr = Some(saddr);
    }

    pub fn get_daddr(&self) -> Address {
        self.daddr.clone()
    }

    pub fn get_saddr(&self) -> Option<Address> {
        self.saddr.clone()
    }

    pub fn ref_daddr(&self) -> &Address {
        &self.daddr
    }

    pub fn ref_saddr(&self) -> &Option<Address> {
        &self.saddr
    }

    pub fn ref_val(&self) -> &T {
        &self.val
    }

    pub fn get_val(&self) -> T {
        self.val.clone()
    }

    pub fn into_val(self) -> T {
        self.val
    }
}

#[derive(Debug, Clone)]
pub(super) enum LastHop {
    Local,
    Router(Address),
}

#[derive(Clone, Debug)]
pub(super) struct BusPacket<T> {
    inner: Packet<T>,
    last_hop: LastHop,
}

impl<T: Clone + Debug> BusPacket<T> {
    pub fn from_local_packet(pkt: Packet<T>) -> Self {
        Self {
            inner: pkt,
            last_hop: LastHop::Local,
        }
    }

    pub fn ref_inner(&self) -> &Packet<T> {
        &self.inner
    }

    pub fn into_local_packet(self) -> Packet<T> {
        self.inner
    }

    pub fn ref_last_hop(&self) -> &LastHop {
        &self.last_hop
    }
}
