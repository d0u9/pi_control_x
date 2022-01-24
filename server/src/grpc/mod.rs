pub mod error;
pub use error::*;

pub mod disk;
pub use disk::*;

mod grpc_api {
    tonic::include_proto!("grpc_api"); // The string specified here must match the proto package name
}
