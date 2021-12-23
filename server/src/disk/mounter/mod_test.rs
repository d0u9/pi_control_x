#[cfg(test)]
use super::*;

#[tokio::test]
async fn mounter_test() {
    let mounter = Builder::new().commit();

    mounter.mount_as_label("/dev/vdb1").unwrap();
    println!("donw");
}

use crate::core::EventEnum;
use crate::event_generator;
use crate::shutdown;
use crate::udev::{Event as UdevEvent, EventType as UdevEventType};
use ::std::path::PathBuf;
use ::tokio::time::{self, Duration};

#[tokio::test]
async fn mounter_poller_test() {
    let bus = bus::Bus::new();

    let event_generator = event_generator::Builder::new()
        .start(Duration::from_secs(1))
        .interval(Duration::from_secs(130))
        .event(EventEnum::Udev(UdevEvent {
            squence_number: 15,
            event_type: UdevEventType::Add,
            syspath: PathBuf::from("/sys/test"),
            devtype: None,
            devnode: Some(PathBuf::from("/dev/vdb1")),
            ..Default::default()
        }))
        .commit()
        .unwrap();
    let generator_poller = event_generator::GeneratorPoller::new(event_generator, bus.clone());

    let (generator_shutsend, generator_shutrecv) = shutdown::new();
    let generator_handler = generator_poller.spawn(generator_shutrecv);

    let mounter = Builder::new().commit();
    let poller = MounterPoller::new(mounter, bus);

    let (shutsend, shutrecv) = shutdown::new();
    let handler = poller.spawn(shutrecv);

    time::sleep(time::Duration::from_secs(3)).await;

    generator_shutsend.shutdown();
    generator_handler.await.unwrap();

    shutsend.shutdown();
    handler.await.unwrap();
}
