#[cfg(test)]
use super::udev::UdevMonitor;
use crate::Shutdown;

#[tokio::test]
async fn udev_test() {
    let mut socket = UdevMonitor::new()
        .expect("Cannot create udev")
        .match_subsystem_devtype("block", "disk")
        .expect("Cannot add subsystem")
        .listen()
        .unwrap()
        ;

    loop {
        let events = socket.read()
            .await
            .unwrap()
            .into_iter()
            .for_each(|x| println!("{:?}", x));
    }
}

use super::UdevPoller;
use ::tokio::time;
use std::time::Duration;

#[tokio::test]
async fn udev_poll_test() {
    let socket = UdevMonitor::new()
        .expect("Cannot create udev")
        .match_subsystem_devtype("block", "disk")
        .expect("Cannot add subsystem")
        .listen()
        .unwrap()
        ;

    let (send, recv) = Shutdown::new();
    let poller = UdevPoller::new(socket);
    let mut events = poller.subscribe();
    let handler = poller.spawn(recv);
    loop {
        ::tokio::select! {
            _ = time::sleep(Duration::from_secs(10)) => { break; }
            Ok(e) = events.recv() => { println!("New Event: {:?}", e); }
        }
    }

    send.shutdown();
    handler.await.unwrap();
}
