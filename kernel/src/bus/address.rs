#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Address {
    addr: String,
}

impl Address {
    pub fn new(addr: &str) -> Address {
        Self {
            addr: addr.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(super) struct RouterAddr {
    last_router: Address,
    src: Address,
}

impl RouterAddr {
    pub(super) fn rt_addr(&self) -> &Address {
        &self.last_router
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(super) enum BusAddress {
    Broadcast,
    Addr(Address),
    Router(RouterAddr),
}
