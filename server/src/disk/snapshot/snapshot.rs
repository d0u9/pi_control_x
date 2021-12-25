use super::Event;
use crate::core::EventEnum;
#[cfg(target_os = "linux")]
use crate::disk::disk_enumerator;
use crate::disk::Disk;
use crate::result::Result;

#[derive(Debug, Default)]
pub struct Builder;

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn commit(self) -> Snapshot {
        Snapshot::default()
    }
}

#[derive(Default)]
pub struct Snapshot {
    disks: Vec<Disk>,
}

impl Snapshot {
    pub fn refresh(&mut self, disks: Vec<Disk>) {
        self.disks = disks;
    }

    pub fn event_process(&mut self, event: EventEnum) -> Result<Option<EventEnum>> {
        match event {
            #[cfg(target_os = "linux")]
            EventEnum::DiskEnumerator(e) => self.event_disk_enumerator(e),
            _ => Ok(None),
        }
    }

    #[cfg(target_os = "linux")]
    fn event_disk_enumerator(
        &mut self,
        event: disk_enumerator::Event,
    ) -> Result<Option<EventEnum>> {
        self.refresh(event.disks.clone());
        let event = EventEnum::Snapshot(Event { disks: event.disks });

        Ok(Some(event))
    }
}
