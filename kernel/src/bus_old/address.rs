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
    pub (super) fn new(last_router: &Address, src: &Address) -> Self {
        RouterAddr {
            last_router: last_router.clone(),
            src: src.clone(),
        }
    }

    pub(super) fn set_last_router(&mut self, addr: &Address) {
        self.last_router = addr.clone();
    }

    pub(super) fn set_src(&mut self, addr: &Address) {
        self.src = addr.clone();
    }

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
