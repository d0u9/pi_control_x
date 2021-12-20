use std::convert::From;
use std::path::{Path, PathBuf};

#[derive(Default, Debug)]
pub struct Disk {
    pub mount_point: PathBuf,
    pub devnode: PathBuf,
    pub label: String,
}

/*
        Mount {
            info: MountInfo {
                id: 337,
                parent: 96,
                dev: DeviceId {
                    major: 253,
                    minor: 17,
                },
                root: "/",
                mount_point: "/mnt/removable/vdb1",
                fs: "/dev/vdb1",
                fs_type: "ext4",
            },
            fs_label: Some(
                "MYDISK",
            ),
            disk: Some(
                Disk {
                    name: "vdb",
                    rotational: Some(
                        true,
                    ),
                    removable: Some(
                        false,
                    ),
                    ram: false,
                    lvm: false,
                    crypted: false,
                },
            ),
            stats: Some(
                Stats {
                    bsize: 4096,
                    blocks: 25671657,
                    bfree: 25656291,
                    bavail: 24341489,
                    files: 6553600,
                    ffree: 6553589,
                    favail: 6553589,
                },
            ),
        },
*/

#[cfg(target_os = "linux")]
impl From<lfs_core::Mount> for Disk {
    fn from(mount: lfs_core::Mount) -> Self {
        Self {
            mount_point: mount.info.mount_point,
            devnode: PathBuf::from(mount.info.fs),
            label: mount.fs_label.unwrap_or(String::from("")),
            ..Default::default()
        }
    }
}
