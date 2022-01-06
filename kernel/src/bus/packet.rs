use super::address::Address;

#[derive(Debug, Clone)]
pub struct Packet<T> {
    val: T,
    src: Address,
    dst: Address,
    last_hop: Address,
}

impl<T> Packet<T> {
}
