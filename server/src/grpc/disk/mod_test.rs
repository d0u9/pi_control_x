use std::fmt::Debug;

use claim::assert_ok;
use futures::future::FutureExt;
use test_log::test;
use tokio::sync::broadcast;
use tokio::time::{self, Duration};
use tonic::transport::Server;

use tokyo_bus::address::Address;
use tokyo_bus::packet_endpoint::{PktEndpoint, PktWire};
use tokyo_bus::switch::{Switch, SwitchHandler, SwitchServer};

use super::lib::*;
use crate::bus_types::*;
mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}

use grpc_api::disk_client::DiskClient;
use grpc_api::{ListRequest, WatchRequest};

fn create_bus<T: Clone + Debug>(
    addr: Address,
    switch_name: &str,
) -> (SwitchServer<T>, SwitchHandler<T>, PktEndpoint<T>) {
    let (ep0, ep1) = PktWire::endpoints(addr);

    let mut switch = Switch::new();
    switch.set_name(switch_name);
    assert_ok!(switch.attach_endpoint(ep0));

    let (server, switch_handler) = assert_ok!(switch.server());

    (server, switch_handler, ep1)
}

#[test(tokio::test)]
async fn test_grpc_disk_list_server() {
    let addr_str = "[::1]:50051";
    let addr = assert_ok!(addr_str.parse());

    let disk_enumerator_addr = Address::new("disk_enumerator");
    let (switch_server, switch_handler, target_ep) =
        create_bus::<BusData>(disk_enumerator_addr.clone(), "list_disk_server");

    let switch_join_handler = tokio::spawn(async move {
        switch_server.serve().await;
    });

    let switch_ctrl = SwitchCtrl::new(switch_handler.clone());
    let service = GrpcDiskApiService::new(switch_ctrl);
    let service = assert_ok!(service.service().await);

    let (shut_tx, mut shut_rx) = broadcast::channel::<()>(1);
    let grpc_join_handler = tokio::spawn(async move {
        assert_ok!(
            Server::builder()
                .add_service(service)
                .serve_with_shutdown(addr, shut_rx.recv().map(|_| ()))
                .await
        );
    });

    let disk_enumerator_join_handler = tokio::spawn(async move {
        let (tx, mut rx) = assert_ok!(target_ep.split());
        let (data, saddr, _) = assert_ok!(rx.recv_tuple().await);
        let msg = format!("echo - {:?}", data);
        assert_ok!(tx.send_data(&saddr, BusData::GrpcDisk(GrpcDiskData { msg })));
    });

    // Wait for server
    time::sleep(Duration::from_millis(5)).await;

    let client = DiskClient::connect(format!("http://{}", addr_str)).await;
    let mut client = assert_ok!(client);

    let rqst = tonic::Request::new(ListRequest {
        timestamp: "11111".to_string(),
    });

    let reps = assert_ok!(client.list(rqst).await);
    println!("xxxxxxxxxxxxxxxxx {:?}", reps);

    assert_ok!(switch_handler.shutdown_server());
    assert_ok!(shut_tx.send(()));

    assert_ok!(switch_join_handler.await);
    assert_ok!(grpc_join_handler.await);
    assert_ok!(disk_enumerator_join_handler.await);
}

#[test(tokio::test)]
async fn test_grpc_disk_watch_server() {
    let addr_str = "[::1]:50052";
    let addr = assert_ok!(addr_str.parse());

    let disk_enumerator_addr = Address::new("disk_enumerator");
    let (switch_server, switch_handler, target_ep) =
        create_bus::<BusData>(disk_enumerator_addr.clone(), "list_disk_server");

    let switch_join_handler = tokio::spawn(async move {
        switch_server.serve().await;
    });

    let switch_ctrl = SwitchCtrl::new(switch_handler.clone());
    let service = GrpcDiskApiService::new(switch_ctrl);
    let service = assert_ok!(service.service().await);

    let (shut_tx, mut shut_rx) = broadcast::channel::<()>(1);
    let grpc_join_handler = tokio::spawn(async move {
        assert_ok!(
            Server::builder()
                .add_service(service)
                .serve_with_shutdown(addr, shut_rx.recv().map(|_| ()))
                .await
        );
    });

    let disk_enumerator_join_handler = tokio::spawn(async move {
        let (tx, mut rx) = assert_ok!(target_ep.split());
        let (data, saddr, _) = assert_ok!(rx.recv_tuple().await);
        let msg = format!("echo - {:?}", data);
        assert_ok!(tx.send_data(
            &saddr,
            BusData::GrpcDisk(GrpcDiskData {
                msg: format!("echo - {}", msg)
            })
        ));

        let disk_watch_addr = Address::new("grpc-disk-watch");
        time::sleep(Duration::from_millis(5)).await;
        assert_ok!(tx.send_data(
            &disk_watch_addr,
            BusData::GrpcDisk(GrpcDiskData {
                msg: format!("echo - {}", msg)
            })
        ));

        time::sleep(Duration::from_millis(5)).await;
        assert_ok!(tx.send_data(
            &disk_watch_addr,
            BusData::GrpcDisk(GrpcDiskData {
                msg: format!("echo - {}", msg)
            })
        ));
    });

    // Wait for server
    time::sleep(Duration::from_millis(5)).await;

    let client = DiskClient::connect(format!("http://{}", addr_str)).await;
    let mut client = assert_ok!(client);

    let rqst = tonic::Request::new(WatchRequest {
        timestamp: "11111".to_string(),
    });

    let reps = assert_ok!(client.watch(rqst).await);
    let mut stream = reps.into_inner();

    for _i in 0..3 {
        if let Some(data) = stream.message().await.unwrap() {
            println!("Received Data {:?}", data);
        }
    }

    assert_ok!(switch_handler.shutdown_server());
    assert_ok!(shut_tx.send(()));

    assert_ok!(switch_join_handler.await);
    assert_ok!(grpc_join_handler.await);
    assert_ok!(disk_enumerator_join_handler.await);
}
