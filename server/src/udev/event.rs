pub use udev::EventType;

use std::path::PathBuf;
use std::ffi::OsString;

#[derive(Default, Clone, Debug)]
pub struct Event {
    pub squence_number: u64,
    pub event_type: EventType,
    pub syspath: PathBuf,
    pub sysname: OsString,
    pub devtype: Option<OsString>,
}

impl std::convert::From<udev::Event> for Event {
    fn from(uevent: udev::Event) -> Self {
        Self {
            squence_number: uevent.sequence_number(),
            event_type: uevent.event_type(),
            syspath: uevent.syspath().to_owned(),
            sysname: uevent.sysname().to_owned(),
            devtype: uevent.devtype().map(|x| x.to_owned()),
        }
    }
}


