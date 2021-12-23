use crate::disk::Disk;

#[derive(Debug, Clone, Default)]
pub struct Event {
    pub disks: Vec<Disk>,
}
