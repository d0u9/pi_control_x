use futures::StreamExt;
use std::time::Duration;
use test_log::test;
use tokio::time;
use claim::assert_ok;

use super::address::*;
use super::packet::*;
use super::wire::*;

#[test(tokio::test)]
async fn test_wire_stream() {
    let (ep0, ep1) = Wire::endpoints::<u32>();

    let (_, ep0_rx) = ep0.split();
    let ep0_rx_stream = RxStream::new(ep0_rx);

    let join = tokio::spawn(async move {
        let (tx, _) = ep1.split();
        time::sleep(Duration::from_millis(10)).await;
        let mut pkt = Packet::new(Address::P2P, 0xdeadbeef_u32);
        pkt.set_saddr(Address::P2P);
        tx.send_pkt(pkt);

        let mut pkt = Packet::new(Address::P2P, 0xdeadbeed_u32);
        pkt.set_saddr(Address::P2P);
        time::sleep(Duration::from_millis(10)).await;
        tx.send_pkt(pkt);
    });

    let result = ep0_rx_stream.collect::<Vec<_>>().await;
    println!("stream result = {:?}", result);
    assert_eq!(result.len(), 2);
    let val0 = assert_ok!(&result[0]).0;
    assert_eq!(val0, 0xdeadbeef_u32);
    let val1 = assert_ok!(&result[1]).0;
    assert_eq!(val1, 0xdeadbeed_u32);

    join.await.unwrap();
}

