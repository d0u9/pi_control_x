use std::convert::From;

use crate::bus_types::BusEndpoint;
use lfs_core::{self, Mount};

#[cfg(test)]
#[path = "implementation_test.rs"]
mod test;

pub struct DefaultFilter {
}

#[derive(Debug)]
pub struct Disk {
    name: String,
    label: String,
    block_size: u64,
    blocks: u64,
    free_blocks: u64,
    available_blocks: u64,
    dev: String,
    fs_type: String,
    mount_point: String,
}

impl From<Mount> for Disk {
    fn from(mount: Mount) -> Self {
        let label = mount.fs_label.unwrap_or_default();

        let mut name = "".to_owned();
        if let Some(disk) = mount.disk {
            name = disk.name;
        }

        let mut block_size = 0;
        let mut blocks = 0;
        let mut free_blocks = 0;
        let mut available_blocks = 0;
        if let Some(stat) = mount.stats {
            block_size = stat.bsize;
            blocks = stat.blocks;
            free_blocks = stat.bfree;
            available_blocks = stat.bavail;
        }

        let mount_point = mount.info.mount_point.to_str().unwrap_or("").to_string();

        Self {
            name,
            label,
            block_size,
            blocks,
            free_blocks,
            available_blocks,
            dev: mount.info.fs,
            fs_type: mount.info.fs_type,
            mount_point,
        }
    }
}

pub struct Enumerator {

}

impl Enumerator {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Enumerator { }
    }

    pub fn enumerate_disk(&self) {
        // NOTE: remove unwrap
        let mounts = lfs_core::read_mounts().unwrap();
        let mounts = mounts.into_iter()
            .filter(|m| !m.info.mount_point.starts_with("/sys/"))
            .filter(|m| !m.info.mount_point.starts_with("/run/"))
            .filter(|m| !m.info.mount_point.starts_with("/proc/"))
            .filter(|m| !m.info.mount_point.starts_with("/var/"))
            .filter(|m| !m.info.mount_point.starts_with("/dev/"))
            .map(Disk::from)
            .collect::<Vec<_>>();

        dbg!("read_mounts: {:?}", mounts);
    }

    pub async fn serve(bus_endpoint: BusEndpoint) {
        let (tx, mut rx) = bus_endpoint.split().unwrap();

        loop {
            tokio::select! {
                request = rx.recv() => {
                    
                }
            }
        }
    }
}

