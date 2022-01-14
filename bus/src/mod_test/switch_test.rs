#[cfg(test)]
use futures::future::FutureExt;
use std::time::Duration;
use test_log::test;
use tokio::sync::mpsc;
use tokio::time;

#[cfg(test)]
use super::address::*;
use super::packet::*;
use super::switch::*;
use super::wire::*;

#[test(tokio::test)]
async fn switch_basic_test() {
    let (epa0, epa1) = Wire::endpoints::<u32>();
    let (epb0, epb1) = Wire::endpoints::<u32>();

    let saddr = Address::new("addr_src");
    let daddr = Address::new("addr_dst");
    let switch = Switch::<u32>::builder()
        .attach(saddr.clone(), epa0)
        .expect("attach endpoint failed")
        .attach(daddr.clone(), epb0)
        .expect("switch attach failed")
        .set_name("switch_basic_test")
        .done();

    let (shut_tx, mut shut_rx) = mpsc::channel::<()>(1);
    let join = tokio::spawn(async move {
        switch.poll_with_graceful(shut_rx.recv().map(|_| ())).await;
    });

    time::sleep(Duration::from_millis(10)).await;

    let (stx, _) = epa1.split();
    let (dtx, mut drx) = epb1.split();
    stx.send(daddr.clone(), 0xdeadbeef);

    let recv_pkt = drx.recv().await.expect("rxb received failed");
    let mut target_pkt = Packet::new(daddr, 0xdeadbeef_u32);
    target_pkt.set_saddr(saddr.clone());

    assert_eq!(recv_pkt.ref_daddr(), target_pkt.ref_daddr());
    assert_eq!(recv_pkt.ref_saddr(), target_pkt.ref_saddr());
    assert_eq!(recv_pkt.ref_val(), target_pkt.ref_val());

    let _make_dest_tx_live_long_enough = dtx;

    time::sleep(Duration::from_millis(10)).await;
    shut_tx.send(()).await.expect("Send shutdown signal failed");

    join.await.expect("join failed");
}

#[test(tokio::test)]
async fn switch_broadcast_test() {
    let (epa0, epa1) = Wire::endpoints::<u32>();
    let (epb0, epb1) = Wire::endpoints::<u32>();
    let (epc0, epc1) = Wire::endpoints::<u32>();

    let a_addr = Address::new("addr_a");
    let b_addr = Address::new("addr_b");
    let c_addr = Address::new("addr_c");

    let switch = Switch::<u32>::builder()
        .attach(a_addr.clone(), epa0)
        .expect("attach endpoint failed")
        .attach(b_addr.clone(), epb0)
        .expect("attach endpoint failed")
        .attach(c_addr.clone(), epc0)
        .expect("attach endpoint failed")
        .set_name("switch_broadcast_test")
        .done();

    let (shut_tx, mut shut_rx) = mpsc::channel::<()>(1);
    let join = tokio::spawn(async move {
        switch.poll_with_graceful(shut_rx.recv().map(|_| ())).await;
    });

    time::sleep(Duration::from_millis(10)).await;

    let (a_tx, mut a_rx) = epa1.split();
    let (b_tx, _) = epb1.split();
    let (c_tx, mut c_rx) = epc1.split();
    b_tx.send(Address::Broadcast, 0xdeadbeef);

    let mut target_pkt = Packet::new(Address::Broadcast, 0xdeadbeef_u32);
    target_pkt.set_saddr(b_addr.clone());

    let a_recv_pkt = a_rx.recv().await.expect("a_rx received failed");
    let c_recv_pkt = c_rx.recv().await.expect("b_rx received failed");

    assert_eq!(a_recv_pkt.ref_daddr(), &Address::Broadcast);
    assert_eq!(a_recv_pkt.ref_saddr(), target_pkt.ref_saddr());
    assert_eq!(a_recv_pkt.ref_val(), target_pkt.ref_val());

    assert_eq!(c_recv_pkt.ref_daddr(), &Address::Broadcast);
    assert_eq!(c_recv_pkt.ref_saddr(), target_pkt.ref_saddr());
    assert_eq!(c_recv_pkt.ref_val(), target_pkt.ref_val());

    let _make_a_tx_live_long_enough = a_tx;
    let _make_c_tx_live_long_enough = c_tx;

    time::sleep(Duration::from_millis(10)).await;
    shut_tx.send(()).await.expect("Send shutdown signal failed");

    join.await.expect("join failed");
}
