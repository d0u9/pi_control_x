use crate::core::EventEnum;
use crate::result::{Error, Result};
use crate::udev;
use ::std::ffi::{OsStr, OsString};
use ::std::fs::create_dir;
use ::std::path::Path;
use sys_mount::{Mount, MountFlags, SupportedFilesystems};

pub struct Builder {}

impl Builder {
    pub fn new() -> Self {
        Self {}
    }

    pub fn commit(self) -> Mounter {
        Mounter {}
    }
}

pub struct Mounter {}

impl Mounter {
    pub fn mount(&self) -> Result<()> {
        Ok(())
    }

    // Create a directory the same name as label, and mount disk there.
    // dev: `/dev/vdb1`
    pub fn mount_as_label(&self, dev: &str) -> Result<()> {
        println!("mount {:?}", dev);
        let label_info = lfs_core::read_labels()?
            .into_iter()
            .find(|x| x.fs_name == dev)
            .ok_or(Error::with_str("cannot find label"))?;

        let dev_path = Path::new(&label_info.fs_name)
            .file_name()
            .ok_or(Error::with_str("label fs_name is wrong"))?;

        let parent = Path::new("/mnt/removable");
        if !parent.is_dir() {
            Error::with_str(&format!("Parent dir [{:?}] doesn't exist", parent));
        }

        let mount_point = parent.join(dev_path);
        create_dir(&mount_point);

        self.do_mount(Path::new(dev), &mount_point)?;

        Ok(())
    }

    pub fn event_process(&self, event: EventEnum) -> Result<Option<EventEnum>> {
        match event {
            EventEnum::Udev(e) => self.event_udev(e),
            _ => Ok(None),
        }
    }

    fn event_udev(&self, event: udev::Event) -> Result<Option<EventEnum>> {
        let dev_node = event
            .devnode
            .ok_or(Error::with_str("Device has no devnode"))?;
        let dev_node = Path::new(&dev_node);
        self.mount_as_label(
            dev_node
                .to_str()
                .ok_or(Error::with_str("Invalid dev path"))?,
        )?;

        Ok(Some(EventEnum::NULL))
    }

    fn do_mount(&self, dev: &Path, target: &Path) -> Result<()> {
        let supported = SupportedFilesystems::new()?;

        let mount_result = Mount::new(dev, target, &supported, MountFlags::empty(), None)?;

        Ok(())
    }
}
