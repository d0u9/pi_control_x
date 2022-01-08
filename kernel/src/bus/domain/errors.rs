use std::convert::From;

use super::super::switch::SwitchError;

#[derive(Debug)]
pub enum DomainError {
    InvalidHandler,
    AddressInUsed,
    AddressInvalid,
}

impl From<SwitchError> for DomainError {
    fn from(e: SwitchError) -> Self {
        match e {
            SwitchError::AddressInUsed => Self::AddressInUsed,
            SwitchError::AddressInvalid => Self::AddressInvalid,
        }
    }
}

