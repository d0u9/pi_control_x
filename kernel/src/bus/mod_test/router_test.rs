#[cfg(test)]

use futures::future::FutureExt;
use std::net::Ipv4Addr;
use std::time::Duration;
use test_log::test;
use tokio::sync::broadcast;
use tokio::time;

use super::address::*;
use super::router::*;
use super::switch::*;
use super::wire::*;

#[test(tokio::test)]
async fn router_create_test() {
    let (ep_send0, ep_send1) = Wire::endpoints::<u32>();
    let (ep_router_a_0, ep_router_a_1) = Wire::endpoints::<u32>();
    let (ep_router_b_0, ep_router_b_1) = Wire::endpoints::<Ipv4Addr>();

    let send_addr = Address::new("send_addr");
    let router_addr_a = Address::new("router_addr_a");

    let switch = Switch::<u32>::builder()
        .attach(send_addr.clone(), ep_send1)
        .expect("attach send addr failed")
        .attach_router(router_addr_a.clone(), ep_router_a_0)
        .expect("attach router a failed")
        .set_gateway(router_addr_a)
        .expect("set gateway failed")
        .done();

    let router = Router::builder()
        .set_name("router_create_test")
        .set_endpoint0(ep_router_a_1)
        .set_endpoint1(ep_router_b_1)
        .done()
        .expect("router build failed");

    let (shut_tx, _) = broadcast::channel::<()>(1);
    let mut shut_switch = shut_tx.subscribe();
    let mut shut_router = shut_tx.subscribe();
    let join_handler = tokio::spawn(async move {
        tokio::select! {
            _ = switch.poll_with_graceful(shut_switch.recv().map(|_|())) => {},
            _ = router.poll_with_graceful(shut_router.recv().map(|_|())) => {},
        }
    });

    // Sleep a few seconds to wait for system boot up.
    time::sleep(Duration::from_millis(10)).await;

    let send_val = 0xac1097d6_u32;
    let (tx, _) = ep_send0.split();
    let (_, mut rx) = ep_router_b_0.split();
    tx.send(Address::new("unknown address"), send_val);

    let target_val = Ipv4Addr::from(send_val);
    let received = rx.recv().await.expect("Received failed");
    println!("received packet: {:?}", received);
    assert_eq!(target_val, received.into_val());

    time::sleep(Duration::from_millis(10)).await;

    shut_tx.send(()).expect("send shutdown signal failed");

    time::sleep(Duration::from_millis(10)).await;

    join_handler.await.expect("thread failed");
}

#[test(tokio::test)]
async fn router_two_switch_test() {
    let (ep_send0, ep_send1) = Wire::endpoints::<u32>();
    let (ep_router_a_0, ep_router_a_1) = Wire::endpoints::<u32>();
    let (ep_router_b_0, ep_router_b_1) = Wire::endpoints::<Ipv4Addr>();
    let (ep_recv0, ep_recv1) = Wire::endpoints::<Ipv4Addr>();

    let send_addr = Address::new("send_addr");
    let router_addr_a = Address::new("router_addr_a");
    let router_addr_b = Address::new("router_addr_b");
    let recv_addr = Address::new("recv_addr");

    let switch1 = Switch::<u32>::builder()
        .attach(send_addr.clone(), ep_send1)
        .expect("attach send addr failed")
        .attach_router(router_addr_a.clone(), ep_router_a_0)
        .expect("attach router a failed")
        .set_gateway(router_addr_a)
        .expect("set gateway failed")
        .set_name("switch1")
        .done();

    let switch2 = Switch::<Ipv4Addr>::builder()
        .attach(recv_addr.clone(), ep_recv1)
        .expect("attach send addr failed")
        .attach_router(router_addr_b.clone(), ep_router_b_0)
        .expect("attach router a failed")
        .set_gateway(router_addr_b)
        .expect("set gateway failed")
        .set_name("switch2")
        .done();

    let router = Router::builder()
        .set_name("router_two_switch_test")
        .set_endpoint0(ep_router_a_1)
        .set_endpoint1(ep_router_b_1)
        .done()
        .expect("router build failed");

    let (shut_tx, _) = broadcast::channel::<()>(1);
    let mut shut_switch1 = shut_tx.subscribe();
    let mut shut_switch2 = shut_tx.subscribe();
    let mut shut_router = shut_tx.subscribe();
    let join_handler = tokio::spawn(async move {
        tokio::select! {
            _ = switch1.poll_with_graceful(shut_switch1.recv().map(|_|())) => {
                println!("swich1 quit");
            },
            _ = switch2.poll_with_graceful(shut_switch2.recv().map(|_|())) => {
                println!("swich2 quit");
            },
            _ = router.poll_with_graceful(shut_router.recv().map(|_|())) => {
                println!("router quit");
            },
        }
    });

    // Sleep a few seconds to wait for system boot up.
    time::sleep(Duration::from_millis(10)).await;

    let send_val = 0xac1097d6_u32;
    let (tx, _) = ep_send0.split();
    let (_, mut rx) = ep_recv0.split();
    tx.send(recv_addr, send_val);

    time::sleep(Duration::from_millis(10)).await;

    let target_val = Ipv4Addr::from(send_val);
    let received = rx.recv().await.expect("Received failed");
    println!("received packet: {:?}", received);
    assert_eq!(target_val, received.into_val());

    time::sleep(Duration::from_millis(10)).await;

    shut_tx.send(()).expect("send shutdown signal failed");

    time::sleep(Duration::from_millis(10)).await;

    join_handler.await.expect("thread failed");
}
