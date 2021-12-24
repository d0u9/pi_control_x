use tonic::Request;
use grpc_api::disk_client::DiskClient;
use grpc_api::ListRequest;

pub mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiskClient::connect("http://localhost:9000").await?;

    let request = Request::new(ListRequest {
        timestamp: "vvvxxxx".to_string(),
    });

    let response = client.list(request).await?;

    println!("RESPONSE={:?}", response);

    Ok(())
}
