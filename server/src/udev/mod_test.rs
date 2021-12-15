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
    let handler = poller.spawn(recv);
    ::tokio::time::sleep(std::time::Duration::from_secs(10)).await;
    send.shutdown();
    handler.await.unwrap();
}
