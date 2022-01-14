use futures::future::FutureExt;
use tonic::transport::Server;
use tokio::sync::mpsc;
use test_log::test;
use claim::assert_ok;

use super::main::*;

#[test(tokio::test)]
async fn test_grpc_disk_server() {
    let addr = assert_ok!("[::1]:50051".parse());
    let service = DiskApiService::new().service();

    let (shut_tx, mut shut_rx) = mpsc::channel::<()>(1);

    let jh = tokio::spawn(async move {
        assert_ok!(Server::builder()
                   .add_service(service)
                   .serve_with_shutdown(addr, shut_rx.recv().map(|_|()))
                   .await);
    });

    assert_ok!(shut_tx.send(()).await);
    assert_ok!(jh.await);
}

