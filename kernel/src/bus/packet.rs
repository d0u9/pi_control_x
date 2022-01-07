#![cfg_attr(test, allow(dead_code))]
use std::convert::From;
use std::fmt::Debug;

use super::address::Address;

#[derive(Debug, Clone)]
pub struct Packet<T> {
    val: T,
    daddr: Address,
    saddr: Option<Address>,
    rt_info: Option<RouteInfo>
}

impl<T: Clone + Debug> Packet<T> {
    pub fn new(daddr: Address, val: T) -> Self {
        Self {
            val,
            daddr,
            saddr: None,
            rt_info: None,
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

    pub fn into<U: From<T>>(self) -> Packet<U> {
        Packet {
            val: self.val.into(),
            daddr: self.daddr,
            saddr: self.saddr,
            rt_info: self.rt_info,
        }
    }
}

impl<T: Clone + Debug> Packet<T> {
    pub(super) fn ref_rt_info(&self) -> &Option<RouteInfo> {
        &self.rt_info
    }
}

#[derive(Debug, Clone)]
pub struct RouteInfo {
    pub(super) last_hop: Address,
}

