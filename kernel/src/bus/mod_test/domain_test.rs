#[cfg(test)]
use std::net::Ipv4Addr;
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

    let server = domain.done();
    tokio::spawn(async move {
        server.serve(shut_rx.recv().map(|_|())).await;
    });

    time::sleep(Duration::from_millis(10)).await;

    let (tx, _) = ep1_0.split();
    let (_, mut rx) = ep1_1.split();

    tx.send(Address::new("ep1"), 0xdeadbeef_u32);

    let (data, saddr, daddr) = rx.recv_data_addr().await.expect("endpoint read failed");

    println!("saddr = {:?}, daddr = {:?}", saddr, daddr);
    assert_eq!(data, 0xdeadbeef_u32);

    shut_tx.send(()).await.expect("send shutdown signal failed");

    time::sleep(Duration::from_millis(10)).await;
}

#[test(tokio::test)]
async fn domain_switch_broadcast_test() {
    let mut domain = Domain::new();
    let switch1 = domain.add_switch::<u32>("switch1");

    let ep1_0 = domain.add_endpoint::<u32>(&switch1, Address::new("ep0")).expect("add_endpoint failed");
    let ep1_1 = domain.add_endpoint::<u32>(&switch1, Address::new("ep1")).expect("add_endpoint failed");
    let ep1_2 = domain.add_endpoint::<u32>(&switch1, Address::new("ep2")).expect("add_endpoint failed");

    let (shut_tx, mut shut_rx) = mpsc::channel::<()>(1);

    let server = domain.done();
    tokio::spawn(async move {
        server.serve(shut_rx.recv().map(|_|())).await;
    });

    time::sleep(Duration::from_millis(10)).await;

    let (tx, _) = ep1_0.split();
    let (_, mut rx1) = ep1_1.split();
    let (_, mut rx2) = ep1_2.split();

    tx.send(Address::Broadcast, 0xdeadbeef_u32);

    let (data, saddr, daddr) = rx1.recv_data_addr().await.expect("endpoint read failed");

    println!("rx1 => saddr = {:?}, daddr = {:?}", saddr, daddr);
    assert_eq!(data, 0xdeadbeef_u32);

    let (data, saddr, daddr) = rx2.recv_data_addr().await.expect("endpoint read failed");
    println!("rx2 => saddr = {:?}, daddr = {:?}", saddr, daddr);
    assert_eq!(data, 0xdeadbeef_u32);

    shut_tx.send(()).await.expect("send shutdown signal failed");

    time::sleep(Duration::from_millis(10)).await;
}

#[test(tokio::test)]
async fn domain_router_test() {
    let mut domain = Domain::new();
    let switch1 = domain.add_switch::<u32>("switch1");
    let switch2 = domain.add_switch::<Ipv4Addr>("switch2");

    let ep1_0 = domain.add_endpoint::<u32>(&switch1, Address::new("switch_ep0")).expect("add_endpoint failed");
    let ep2_0 = domain.add_endpoint::<Ipv4Addr>(&switch2, Address::new("switch_ep1")).expect("add_endpoint failed");


    domain.join_switches::<u32, Ipv4Addr>(&switch1, &switch2, "router").expect("Join switch failed");

    let (shut_tx, mut shut_rx) = mpsc::channel::<()>(1);

    domain.draw();
    let server = domain.done();
    tokio::spawn(async move {
        server.serve(shut_rx.recv().map(|_|())).await;
    });

    time::sleep(Duration::from_millis(10)).await;

    let (tx, _) = ep1_0.split();
    let (_, mut rx) = ep2_0.split();

    tx.send(Address::new("switch_ep1"), 0xdeadbeef_u32);

    let (data, saddr, daddr) = rx.recv_data_addr().await.expect("endpoint read failed");

    println!("saddr = {:?}, daddr = {:?}", saddr, daddr);
    assert_eq!(u32::from(data), 0xdeadbeef_u32);

    shut_tx.send(()).await.expect("send shutdown signal failed");

    time::sleep(Duration::from_millis(10)).await;
}


#[test(tokio::test)]
async fn domain_2router_test() {
    let mut domain = Domain::new();
    let switch1 = domain.add_switch::<u32>("switch1");
    let switch2 = domain.add_switch::<Ipv4Addr>("switch2");
    let switch3 = domain.add_switch::<u32>("switch3");
    let switch4 = domain.add_switch::<u32>("switch4");

    let ep1_0 = domain.add_endpoint::<u32>(&switch1, Address::new("switch_ep0")).expect("add_endpoint failed");
    let ep2_0 = domain.add_endpoint::<Ipv4Addr>(&switch2, Address::new("switch_ep1")).expect("add_endpoint failed");
    let ep3_0 = domain.add_endpoint::<u32>(&switch3, Address::new("switch_ep2")).expect("add_endpoint failed");
    let ep4_0 = domain.add_endpoint::<u32>(&switch4, Address::new("switch_ep4")).expect("add_endpoint failed");


    domain.join_switches::<u32, Ipv4Addr>(&switch1, &switch2, "router1-2").expect("Join switch failed");
    domain.join_switches::<Ipv4Addr, u32>(&switch2, &switch3, "router2-3").expect("Join switch failed");
    domain.join_switches::<Ipv4Addr, u32>(&switch2, &switch4, "router2-4").expect("Join switch failed");

    let (shut_tx, mut shut_rx) = mpsc::channel::<()>(1);

    domain.draw();
    let server = domain.done();
    tokio::spawn(async move {
        server.serve(shut_rx.recv().map(|_|())).await;
    });

    time::sleep(Duration::from_millis(10)).await;

    let (tx, _) = ep1_0.split();
    let (_, mut rx) = ep4_0.split();

    tx.send(Address::new("switch_ep4"), 0xdeadbeef_u32);

    let (data, saddr, daddr) = rx.recv_data_addr().await.expect("endpoint read failed");

    println!("saddr = {:?}, daddr = {:?}", saddr, daddr);
    assert_eq!(data, 0xdeadbeef_u32);

    shut_tx.send(()).await.expect("send shutdown signal failed");

    time::sleep(Duration::from_millis(10)).await;
}

