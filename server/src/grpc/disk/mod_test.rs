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

use super::lib::*;

mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}

use grpc_api::disk_client::DiskClient;
use grpc_api::ListRequest;

fn create_bus<T: Clone + Debug>(target_addr: Address) -> (Switch<T>, Endpoint<T>) {
    let (ep0, ep1) = Wire::endpoints::<T>();

    let switch = Switch::<T>::builder()
        .attach(target_addr, ep0)
        .expect("attach endpoint failed")
        .set_name("switch_broadcast_test")
        .done();

    (switch, ep1)
}

#[test(tokio::test)]
async fn test_grpc_disk_server() {
    let addr_str = "[::1]:50051";
    let addr = assert_ok!(addr_str.parse());

    let (shut_tx, mut shut_rx) = broadcast::channel::<()>(1);

    let target_addr = Address::new("disk_enumerator");

    let (mut switch, target_ep) = create_bus::<DiskBusData>(target_addr.clone());
    let switch_ctrl = switch.get_control_endpoint();

    let jh_switch = tokio::spawn(async move {
        switch.poll_with_graceful(shut_rx.recv().map(|_| ())).await;
    });

    let service = DiskApiService::new(switch_ctrl);
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
        let (tx, mut rx) = target_ep.split();
        let (data, saddr, _) = assert_ok!(rx.recv_data_addr().await);
        let msg = format!("echo - {:?}", data);
        tx.send(saddr, DiskBusData{ msg });
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

