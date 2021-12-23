use crate::core::EventEnum;
use crate::disk::Disk;
use crate::result::{Error, Result};
use crate::disk::mounter;
use ::std::path::{Path, PathBuf};
use super::Event;

#[derive(Debug, Default)]
pub struct Builder {
    mount_point_prefix: PathBuf,
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn mount_point_prefix(mut self, prefix: impl AsRef<Path>) -> Self {
        self.mount_point_prefix = prefix.as_ref().to_owned();
        self
    }

    pub fn commit(self) -> DiskEnumerator {
        DiskEnumerator {
            mount_point_prefix: self.mount_point_prefix,
        }
    }
}

pub struct DiskEnumerator {
    mount_point_prefix: PathBuf,
}

impl DiskEnumerator {
    // Doesn't go over filter
    pub fn get_all(&self) -> Result<Vec<Disk>> {
        let mounts = ::lfs_core::read_mounts()?
            .into_iter()
            .map(|m| m.into())
            .collect::<Vec<_>>();
        Ok(mounts)
    }

    // Applied filter on
    pub fn get(&self) -> Result<Vec<Disk>> {
        let mounts = self.get_all()?;
        let result = mounts
            .into_iter()
            .filter(|x| x.mount_point.starts_with(&self.mount_point_prefix))
            .collect::<Vec<_>>();
        Ok(result)
    }

    pub fn event_process(&self, event: EventEnum) -> Result<Option<EventEnum>> {
        match event {
            EventEnum::Mounter(e) => self.event_mounter(e),
            _ => Ok(None),
        }
    }


    fn event_mounter(&self, _event: mounter::Event) -> Result<Option<EventEnum>> {
        let disks = self.get()?;
        let event = EventEnum::DiskEnumerator(Event{ disks });

        Ok(Some(event))
    }
}
