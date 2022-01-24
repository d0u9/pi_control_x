use std::convert::From;
use std::fmt::Debug;

use tonic::Status;

use crate::bus_types::BusError;

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
    pub fn internal(msg: &str) -> Self {
        Self {
            kind: ErrKind::InternalError,
            msg: msg.to_owned(),
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

impl From<BusError> for GrpcError {
    fn from(err: BusError) -> Self {
        Self {
            kind: ErrKind::BusError,
            msg: format!("Bus error: {:?}", err),
        }
    }
}
