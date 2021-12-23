use crate::disk::Disk;

#[derive(Default, Clone, Debug)]
pub struct Event {
    pub disks: Vec<Disk>,
}
