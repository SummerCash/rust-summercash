use super::{
    core::{
        sys::system::ExecutionError,
        types::{graph::OperationError, transaction::SignatureError},
    },
    p2p::client::{CommunicationError, ConstructionError},
};

pub mod address; // Export the address types & utilities module
pub mod fink; // Export the fink unit conversion utilities module
pub mod io; // Export the io definitions module

/// A generic all-purpose SummerCash error.
#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "a communication error occurred: {}", error)]
    CommunicationError { error: CommunicationError },
    #[fail(display = "an execution error occurred: {}", error)]
    ExecutionError { error: ExecutionError },
    #[fail(display = "an operation error occurred: {}", error)]
    OperationError { error: OperationError },
    #[fail(display = "a signature error occurred: {}", error)]
    SignatureError { error: SignatureError },
    #[fail(display = "a construction error occurred: {}", error)]
    ConstructionError { error: ConstructionError },
}
