#[cfg(test)]
use std::time::Duration;
use futures::FutureExt;
use test_log::test;
use tokio::sync::mpsc;
use tokio::time;

use super::address::Address;
use super::super::domain::*;

#[test(tokio::test)]
async fn domain_create_test() {
    let mut domain = Domain::new();
    let switch1 = domain.add_switch::<u32>("switch1");
    let ep1_0 = domain.add_endpoint::<u32>(&switch1, Address::new("ep0")).expect("add_endpoint failed");
    let ep1_1 = domain.add_endpoint::<u32>(&switch1, Address::new("ep1")).expect("add_endpoint failed");

    let (shut_tx, mut shut_rx) = mpsc::channel::<()>(1);
    tokio::spawn(async move {
        domain.serve(shut_rx.recv().map(|_|())).await;
    });

    time::sleep(Duration::from_millis(10)).await;
}
