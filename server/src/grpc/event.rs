use super::disk::EventDiskList;

#[derive(Clone, Debug)]
pub enum Event {
    Disk(EventDiskList),
}
