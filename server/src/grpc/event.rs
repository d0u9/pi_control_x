use super::disk::DiskListEvent;

#[derive(Clone, Debug)]
pub enum Event {
    DiskList(DiskListEvent),
}
