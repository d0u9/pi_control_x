use std::convert::From;
use std::fmt::Debug;

use tonic::Status;

pub type GrpcResult<T> = Result<T, GrpcError>;

#[derive(Debug, Clone, Copy)]
pub enum ErrKind {
    BusError,
    InternalError,
}

#[derive(Debug, Clone)]
pub struct GrpcError {
    kind: ErrKind,
    msg: String,
}

impl GrpcError {
    pub fn bus_err(e: impl Debug) -> Self {
        Self {
            kind: ErrKind::BusError,
            msg: format!("Bus Error: {:?}", e),
        }
    }
}

impl From<GrpcError> for Status {
    fn from(err: GrpcError) -> Self {
        match err.kind {
            ErrKind::BusError => Status::failed_precondition(err.msg),
            _ => Status::internal(err.msg),
        }
    }
}

