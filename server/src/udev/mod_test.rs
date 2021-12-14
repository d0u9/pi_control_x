#[cfg(test)]
use ::tokio::sync::broadcast;
use super::udev::Udev;

#[tokio::test]
async fn udev_test() {
    let mut socket = Udev::new()
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
