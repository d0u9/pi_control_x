#[cfg(test)]
use std::time::Duration;
use test_log::test;
use tokio::time;
use tokio::sync::mpsc;
use futures::future::FutureExt;

#[cfg(test)]
use super::switch::*;
use super::wire::*;
use super::address::*;

#[test(tokio::test)]
async fn hello_test() {

    let (ep0, ep1) = Wire::new::<u32>();

    let addr = Address::new("addr1");
    let switch = Switch::<u32>::builder()
        .attach(addr.clone(), ep0)
        .expect("switch attach failed")
        .done();

    let (shut_tx, mut shut_rx) = mpsc::channel::<()>(1);
    let join = tokio::spawn(async move {
        switch.poll(shut_rx.recv().map(|_|())).await;
    });

    time::sleep(Duration::from_millis(10)).await;

    let (tx, _) = ep1.split();
    tx.send(12);

    time::sleep(Duration::from_millis(10)).await;
    shut_tx.send(()).await.expect("Send shutdown signal failed");

    join.await.expect("join failed");
}


