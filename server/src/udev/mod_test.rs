#[cfg(test)]
use ::tokio::sync::broadcast;
use super::udev::Udev;
use super::ThreadUdev;

#[tokio::test]
async fn udev_test() {
    let (tx, _) = broadcast::channel(16);
    let _event_chan = Udev::new().expect("Cannot create udev")
        .match_subsystem_devtype("block", "disk").expect("Cannot add subsystem")
        .listen(tx)
        .unwrap()
        ;
}

#[tokio::test]
async fn udev_thread_test() {
    let mut rx = ThreadUdev::new().unwrap()
        .match_subsystem_devtype("block", "disk")
        .expect("Cannot add subsystem")
        .spawn_run()
        .expect("Cannot run")
        ;
    println!("udev_thread_test");
    rx.recv().await.unwrap();
}
