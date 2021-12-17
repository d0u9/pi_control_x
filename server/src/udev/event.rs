pub use udev::EventType;

use libc::dev_t;
use std::ffi::OsString;
use std::path::PathBuf;

#[derive(Default, Clone, Debug)]
pub struct Event {
    pub squence_number: u64,
    pub event_type: EventType,
    pub syspath: PathBuf,
    pub sysname: OsString,
    pub devtype: Option<OsString>,
    pub devnum: Option<dev_t>,
    pub devpath: OsString,
    pub devnode: Option<PathBuf>,
    pub action: Option<OsString>,
}

// Unused udev-rs functions:
//
// &self.is_initialized()
// &self.subsystem()
// &self.sysnum()
// &self.driver()
// &self.parent()
impl std::convert::From<udev::Event> for Event {
    fn from(uevent: udev::Event) -> Self {
        Self {
            squence_number: uevent.sequence_number(),
            event_type: uevent.event_type(),
            syspath: uevent.syspath().to_owned(),
            sysname: uevent.sysname().to_owned(),
            devtype: uevent.devtype().map(|x| x.to_owned()),
            devnum: uevent.devnum(),
            devpath: uevent.devpath().to_owned(),
            devnode: uevent.devnode().map(|v| v.to_owned()),
            action: uevent.action().map(|v| v.to_owned()),
        }
    }
}
