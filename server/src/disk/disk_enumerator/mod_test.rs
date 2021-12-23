#[cfg(test)]
use super::*;

#[tokio::test]
async fn disk_enumerator_test() {
    let enumerator = Builder::new().mount_point_prefix("/mnt").commit();

    let all_mounts = enumerator.get().unwrap();
    dbg!(all_mounts);
}

use crate::event_generator;
use crate::core::EventEnum;
use crate::shutdown;
use crate::disk::mounter::Event as MounterEvent;
use ::tokio::time::{self, Duration};

#[tokio::test]
async fn disk_enumerator_poller_test() {
    let bus = bus::Bus::new();

    let event_generator = event_generator::Builder::new()
        .start(Duration::from_secs(1))
        .interval(Duration::from_secs(130))
        .event(EventEnum::Mounter(MounterEvent {
            ..Default::default()
        }))
        .commit()
        .unwrap();

    let generator_poller = event_generator::GeneratorPoller::new(event_generator, bus.clone());

    let (generator_shutsend, generator_shutrecv) = shutdown::new();
    let generator_handler = generator_poller.spawn(generator_shutrecv);

    let disk_enumerator = Builder::new().mount_point_prefix("/media").commit();
    let poller = DiskEnumeratorPoller::new(disk_enumerator, bus);

    let (shutsend, shutrecv) = shutdown::new();
    let handler = poller.spawn(shutrecv);

    time::sleep(time::Duration::from_secs(3)).await;

    generator_shutsend.shutdown();
    generator_handler.await.unwrap();

    shutsend.shutdown();
    handler.await.unwrap();
}
