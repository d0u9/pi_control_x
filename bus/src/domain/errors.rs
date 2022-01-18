use std::convert::From;

use super::super::switch::SwitchError;
use super::super::router::RouterError;

#[derive(Debug)]
pub enum DomainError {
    InvalidHandler,
    AddressInUsed,
    AddressInvalid,
    SwitchJoinError,
    HandlerIsNotSwitch,
    HandlerIsNotRouter,
    TypeMismatch,
    UnknowCtrlErr,
}

impl From<SwitchError> for DomainError {
    fn from(e: SwitchError) -> Self {
        match e {
            SwitchError::AddressInUsed => Self::AddressInUsed,
            SwitchError::AddressInvalid => Self::AddressInvalid,
            SwitchError::UnknowCtrlErr => Self::UnknowCtrlErr,
        }
    }
}

impl From<RouterError> for DomainError {
    fn from(e: RouterError) -> Self {
        match e {
            RouterError::BuildError => Self::SwitchJoinError,
        }
    }
}

