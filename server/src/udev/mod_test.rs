#[cfg(test)]
use ::tokio::sync::broadcast;
use super::udev::UdevMonitor;

#[tokio::test]
async fn udev_test() {
    let mut socket = UdevMonitor::new()
        .expect("Cannot create udev")
        .match_subsystem_devtype("block", "disk")
        .expect("Cannot add subsystem")
        .listen()
        .unwrap()
        ;

    println!("--------");
    loop {
        let events = socket.read()
            .await
            .unwrap()
            .into_iter()
            .for_each(|x| println!("{:?}", x));
        println!("?????");
    }
}

use super::UdevPoller;

#[tokio::test]
async fn udev_poll_test() {
    let mut socket = UdevMonitor::new()
        .expect("Cannot create udev")
        .match_subsystem_devtype("block", "disk")
        .expect("Cannot add subsystem")
        .listen()
        .unwrap()
        ;

    let poller = UdevPoller::new(socket);
    poller.spawn();
}
