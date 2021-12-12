#[cfg(test)]
mod mod_test;

use crate::result::Result;
use std::ffi::OsStr;

use futures_util::future::ready;
use futures_util::stream::StreamExt;
use tokio_udev::{AsyncMonitorSocket, MonitorBuilder};

struct Udev {
    monitor: AsyncMonitorSocket,
}

impl Udev {
    fn new<T, U>(subsystem: T, devtype: U) -> Result<Self>
    where
        T: AsRef<OsStr>,
        U: AsRef<OsStr>,
    {
        let builder = MonitorBuilder::new()
            .expect("Cannot create monitor builder")
            .match_subsystem_devtype(&subsystem, devtype)
            .expect("xx");
        let builder = builder
            .match_subsystem_devtype(&subsystem, "disk")
            .expect("Filed to add filter");

		let monitor: AsyncMonitorSocket = builder
			.listen()
			.expect("Couldn't create MonitorSocket")
			.try_into()
			.expect("Couldn't create AsyncMonitorSocket");

        Ok(Udev { monitor })
    }

    async fn listen(self) {
        self.monitor
        	.for_each(|event| {
        		if let Ok(event) = event {
        			println!(
        				"Hotplug event: {}: {}",
        				event.event_type(),
        				event.device().syspath().display(),
        				);
        		}
        		ready(())
        	})
        .await;
    }
}
