#[cfg(test)]
use super::*;

use ::tokio::time::Duration;

#[tokio::test]
async fn bus_test() {
    let sleep_time = 500;

    let mut bus = Bus::<String>::new("root");
    let mut endpoint1 = bus.crate_endpoint(&Address::new("ep1"));
    let mut endpoint2 = bus.crate_endpoint(&Address::new("ep2"));
    let (tx, rx) = tokio::sync::mpsc::channel::<()>(2);
    let join_handler = tokio::spawn(async move {
        bus.poll(rx).await;
    });

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    endpoint2.send(&Address::new("ep1"), "VVVVVVV".to_string());

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    endpoint1.send(&Address::new("ep2"), "VVVVVVV1".to_string());

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    dbg!(endpoint1.recv().await);
    dbg!(endpoint2.recv().await);

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;
    tx.send(()).await.unwrap();

    join_handler.await.unwrap();
    println!("---------");
}
