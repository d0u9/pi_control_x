#[cfg(test)]
use super::*;

use crate::shutdown;

#[tokio::test]
async fn grpc_test() {
    let grpc_server = Builder::new()
        .address("127.0.0.1:9000").unwrap()
        .commit()
        .unwrap();
    let (_, mut tx) = shutdown::new();
    grpc_server.serve(tx.wait()).await.unwrap();
}
