use tokyo_bus::address::Address;

pub struct AddrBook;
impl AddrBook {
    pub fn grpc_disk_watch() -> Address {
        Address::new("grpc-disk-watch")
    }

    pub fn disk_enumerator() -> Address {
        Address::new("disk-enumerator")
    }
}
