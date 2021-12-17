use ::std::path::{Path, PathBuf};
use ::lfs_core::{self, Mount};
use crate::result::{Result, Error};

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
    pub fn get_all(&self) -> Result<Vec<Mount>> {
        let mounts = lfs_core::read_mounts()?;
        Ok(mounts)
    }

    // Applied filter on
    pub fn get(&self) -> Result<Vec<Mount>> {
        let mounts = self.get_all()?;
        let result = mounts.into_iter()
            .filter(|x| x.info.mount_point.starts_with(&self.mount_point_prefix))
            .collect::<Vec<_>>();
        Ok(result)
    }
}


