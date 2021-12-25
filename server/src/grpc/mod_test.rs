#[cfg(test)]
use super::*;

use crate::shutdown;

#[tokio::test]
async fn grpc_test() {
    let grpc_server = Builder::new()
        .address("127.0.0.1:9000")
        .unwrap()
        .commit()
        .unwrap();
    let (_, mut tx) = shutdown::new();
    grpc_server.server.serve(tx.wait()).await.unwrap();
}

use crate::dummy_event::responder as event_responder;
use ::tokio::time;

#[tokio::test]
async fn grpc_poller_response_test() {
    let bus = bus::Bus::new();

    let event_responder = event_responder::Builder::new()
        .event_process(|_event| None)
        .commit();

    let responder_poller = event_responder::ResponderPoller::new(event_responder, bus.clone());

    let (responder_shutsend, responder_shutrecv) = shutdown::new();
    let responder_hander = responder_poller.spawn(responder_shutrecv);

    let grpc_server = Builder::new()
        .address("0.0.0.0:9000")
        .unwrap()
        .commit()
        .unwrap();
    let poller = GrpcPoller::new(grpc_server, bus);

    let (shutsend, shutrecv) = shutdown::new();
    let handler = poller.spawn(shutrecv);

    time::sleep(time::Duration::from_secs(10)).await;

    responder_shutsend.shutdown();
    responder_hander.await.unwrap();

    shutsend.shutdown();
    handler.await.unwrap();
}
