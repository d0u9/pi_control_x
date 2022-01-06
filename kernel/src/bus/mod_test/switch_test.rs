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
use super::packet::*;

#[test(tokio::test)]
async fn hello_test() {

    let (epa0, epa1) = Wire::new::<u32>();
    let (epb0, epb1) = Wire::new::<u32>();

    let saddr = Address::new("addr_src");
    let daddr = Address::new("addr_dst");
    let switch = Switch::<u32>::builder()
        .attach(saddr.clone(), epa0)
        .expect("attach endpoint failed")
        .attach(daddr.clone(), epb0)
        .expect("switch attach failed")
        .done();

    let (shut_tx, mut shut_rx) = mpsc::channel::<()>(1);
    let join = tokio::spawn(async move {
        switch.poll(shut_rx.recv().map(|_|())).await;
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


