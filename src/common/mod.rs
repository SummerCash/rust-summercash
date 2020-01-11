use super::p2p::client::CommunicationError;

pub mod address; // Export the address types & utilities module
pub mod fink; // Export the fink unit conversion utilities module
pub mod io; // Export the io definitions module

pub enum Error {
    CommunicationError {
        error: CommunicationError, 
    }
}
