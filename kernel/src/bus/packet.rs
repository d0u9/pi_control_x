use std::fmt::Debug;

use super::address::Address;
use super::wire::Rx;

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
