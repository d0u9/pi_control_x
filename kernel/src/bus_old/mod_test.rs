#[cfg(test)]
use super::router::*;
use super::bus::*;
use super::address::*;

use ::futures::future::FutureExt;
use ::tokio::time::Duration;

#[tokio::test]
async fn bus_test() {
    let sleep_time = 300;

    let mut bus = Bus::<String>::new("root");
    let mut endpoint1 = bus.create_endpoint(&Address::new("ep1"));
    let mut endpoint2 = bus.create_endpoint(&Address::new("ep2"));
    let (tx, mut rx) = tokio::sync::mpsc::channel::<()>(2);
    let join_handler = tokio::spawn(async move {
        bus.serve(rx.recv().map(|_| ())).await;
    });

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    endpoint2.send(&Address::new("ep1"), "VVVVVVV".to_string());

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    endpoint1.send(&Address::new("ep2"), "VVVVVVV1".to_string());

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    dbg!(endpoint1.recv().await);
    dbg!(endpoint2.recv().await);

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    endpoint1.broadcast("hahaha".to_string());

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    dbg!(endpoint2.recv().await);

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    tx.send(()).await.unwrap();

    join_handler.await.unwrap();
    println!("---------");
}

#[tokio::test]
async fn router_create_test() {
    let _router = Builder::<i32, i32>::new().mode(RouterMode::FLAT).create();
}

use ::std::net::Ipv4Addr;

#[tokio::test]
async fn router_func_test() {
    let mut router = Builder::<Ipv4Addr, u32>::new()
        .mode(RouterMode::FLAT)
        .create();

    let mut parent_bus = Bus::<u32>::new("parent");
    let mut local_bus = Bus::<Ipv4Addr>::new("local");

    let parent_router_address = Address::new("parent_router");
    let parent_router_endpoint = parent_bus.create_endpoint(&parent_router_address);
    parent_bus.set_gateway(&parent_router_address);

    let local_router_address = Address::new("local_router");
    let local_router_endpoint = local_bus.create_endpoint(&local_router_address);
    local_bus.set_gateway(&local_router_address);

    router.join(local_router_endpoint, parent_router_endpoint);

    let local_ep1_addr = Address::new("local_ep1");
    let mut local_ep1 = local_bus.create_endpoint(&local_ep1_addr);

    let parent_ep1_addr = Address::new("parent_ep1");
    let mut parent_ep1 = parent_bus.create_endpoint(&parent_ep1_addr);

    let (local_tx, mut local_rx) = tokio::sync::mpsc::channel::<()>(2);
    let (parent_tx, mut parent_rx) = tokio::sync::mpsc::channel::<()>(2);
    let (router_tx, router_rx) = tokio::sync::mpsc::channel::<()>(2);
    let join_handler = tokio::spawn(async move {
        tokio::join! {
            local_bus.serve(local_rx.recv().map(|_|())),
            parent_bus.serve(parent_rx.recv().map(|_|())),
            router.poll(router_rx),
        }
    });

    let sleep_time = 300;
    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    local_ep1.send(&parent_ep1_addr, Ipv4Addr::new(172, 16, 0, 1));

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;
    dbg!(parent_ep1.recv().await);

    tokio::time::sleep(Duration::from_millis(sleep_time)).await;

    router_tx.send(()).await.unwrap();
    local_tx.send(()).await.unwrap();
    parent_tx.send(()).await.unwrap();

    join_handler.await.unwrap();
}
