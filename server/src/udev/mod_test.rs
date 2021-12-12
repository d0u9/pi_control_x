#[cfg(test)]
use super::*;

#[tokio::test]
async fn basic_test() {
    let udev = Udev::new("block", "partition").expect("Cannot create udev");
    udev.listen().await;
}
