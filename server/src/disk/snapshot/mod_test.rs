#[cfg(test)]
use super::*;

#[tokio::test]
async fn disk_snapshot_test() {
    let mut snapshot = Builder::new().commit();
    snapshot.refresh(Vec::new());
}

use crate::core::EventEnum;
use crate::disk::disk_enumerator::Event as DiskEnumeratorEvent;
use crate::disk::Disk;
use crate::event_generator;
use crate::shutdown;
use ::tokio::time::{self, Duration};
use std::path::PathBuf;

#[tokio::test]
async fn disk_snapshot_poller_test() {
    let bus = bus::Bus::new();

    let event_generator = event_generator::Builder::new()
        .start(Duration::from_secs(1))
        .interval(Duration::from_secs(130))
        .event(EventEnum::DiskEnumerator(DiskEnumeratorEvent {
            disks: vec![Disk {
                mount_point: PathBuf::from("/media/doug/UNTITLED"),
                devnode: PathBuf::from("/dev/sdb1"),
                label: String::from("UNTITLED"),
            }],
        }))
        .commit()
        .unwrap();

    let generator_poller = event_generator::GeneratorPoller::new(event_generator, bus.clone());

    let (generator_shutsend, generator_shutrecv) = shutdown::new();
    let generator_handler = generator_poller.spawn(generator_shutrecv);

    let snapshot = Builder::new().commit();
    let poller = SnapshotPoller::new(snapshot, bus);

    let (shutsend, shutrecv) = shutdown::new();
    let handler = poller.spawn(shutrecv);

    time::sleep(time::Duration::from_secs(3)).await;

    generator_shutsend.shutdown();
    generator_handler.await.unwrap();

    shutsend.shutdown();
    handler.await.unwrap();
}
