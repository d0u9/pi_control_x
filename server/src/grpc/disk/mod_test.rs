use std::time::Duration;
use std::fmt::Debug;

use futures::future::FutureExt;
use tonic::transport::Server;
use tokio::time;
use tokio::sync::broadcast;
use test_log::test;
use claim::assert_ok;
use bus::wire::Wire;
use bus::address::Address;
use bus::switch::Switch;
use bus::wire::Endpoint;

use super::main::*;

mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}

use grpc_api::disk_client::DiskClient;
use grpc_api::ListRequest;

fn create_bus<T: Clone + Debug>(target_addr: Address, local_addr: Address) -> (Switch<T>, Endpoint<T>, Endpoint<T>) {
    let (epa0, epa1) = Wire::endpoints::<T>();
    let (epb0, epb1) = Wire::endpoints::<T>();

    let switch = Switch::<T>::builder()
        .attach(target_addr, epa0)
        .expect("attach endpoint failed")
        .attach(local_addr, epb0)
        .expect("attach endpoint failed")
        .set_name("switch_broadcast_test")
        .done();

    (switch, epa1, epb1)
}

#[test(tokio::test)]
async fn test_grpc_disk_server() {
    let addr_str = "[::1]:50051";
    let addr = assert_ok!(addr_str.parse());

    let (shut_tx, mut shut_rx) = broadcast::channel::<()>(1);

    let local_addr = Address::new("grpc_disk");
    let target_addr = Address::new("disk_enumerator");

    let (switch, local_ep, target_dp) = create_bus::<DiskBusData>(local_addr.clone(), target_addr.clone());

    let jh_switch = tokio::spawn(async move {
        switch.poll_with_graceful(shut_rx.recv().map(|_| ())).await;
    });

    let mut service = DiskApiService::new();
    service.attach_bus(local_ep);
    let service = service.service();
    let mut shut_rx = shut_tx.subscribe();
    let jh_server = tokio::spawn(async move {
        assert_ok!(Server::builder()
                   .add_service(service)
                   .serve_with_shutdown(addr, shut_rx.recv().map(|_|()))
                   .await);
    });

    time::sleep(Duration::from_millis(5)).await;

    let jh_disk_enumerator = tokio::spawn(async move {
        let (tx, mut rx) = target_dp.split();
        let data = assert_ok!(rx.recv_data().await);
        let msg = format!("echo - {:?}", data);
        tx.send(local_addr.clone(), DiskBusData{ msg });
    });

    let client = DiskClient::connect(format!("http://{}", addr_str)).await;
    let mut client = assert_ok!(client);

    let rqst = tonic::Request::new(ListRequest {
        timestamp: "11111".to_string(),
    });

    let reps = assert_ok!(client.list(rqst).await);
    println!("xxxxxxxxxxxxxxxxx {:?}", reps);

    assert_ok!(shut_tx.send(()));

    assert_ok!(jh_server.await);
    assert_ok!(jh_switch.await);
    assert_ok!(jh_disk_enumerator.await);
}

