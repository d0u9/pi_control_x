#[cfg(test)]
use super::*;

use ::tokio::time::Duration;

#[tokio::test]
async fn bus_test() {
    let mut bus = Bus::<String>::new("root");
    let endpoint1 = bus.crate_endpoint(Address::new("zzz"));
    let endpoint2 = bus.crate_endpoint(Address::new("zzz2"));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(2);
    let join_handler = tokio::spawn(async move {
        bus.poll(rx).await;
    });

    tokio::time::sleep(Duration::from_secs(2)).await;

    endpoint2.send("VVVVVVV".to_string());

    tokio::time::sleep(Duration::from_secs(2)).await;

    endpoint2.send("VVVVVVV1".to_string());

    tokio::time::sleep(Duration::from_secs(2)).await;
    tx.send(()).await.unwrap();


    join_handler.await;
    println!("---------");
}
