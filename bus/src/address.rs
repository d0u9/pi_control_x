use std::fmt::Display;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Address {
    P2P,
    Broadcast,
    Addr(String),
}

impl Address {
    pub fn new(name: &str) -> Self {
        Address::Addr(name.to_owned())
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::P2P => write!( f, "P2P"),
            Self::Broadcast => write!(f, "BROADCAST"),
            Self::Addr(addr) => write!(f, "{}", addr),
        }
    }
}
