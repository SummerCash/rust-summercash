use super::network; // Import the network module
use serde::{Deserialize, Serialize}; // Import serde serialization

/// A SummerCash network message.
#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    /// The message header
    pub header: Header,

    /// Data sent in message
    pub data: Vec<u8>,
}

/// Implement some message helper methods.
impl Message {
    /// Initialize a new message with the particular header and body.
    pub fn new(header: Header, data: Vec<u8>) -> Message {
        Message {
            header, // Set header
            data,   // Set data
        } // Return initialized message
    }
}

/// A generic message header.
#[derive(Clone, Serialize, Deserialize)]
pub struct Header {
    /// The pseudo-HTTP method associated with the message
    pub method: Method,

    /// The target runtime field
    pub param_name: String,

    /// The networks to broadcast the message to
    pub networks: Vec<network::Network>,
}

/// Implement some header helper methods.
impl Header {
    /// Initialize a new header with the given parameters.
    pub fn new(param_name: &str, method: Method, networks: Vec<network::Network>) -> Header {
        Header {
            method,                            // Set method
            param_name: param_name.to_owned(), // Set parameter name
            networks,                          // Set networks
        } // Return initialized header
    }
}

/// A set of available message methods.
#[derive(Clone, Serialize, Deserialize)]
pub enum Method {
    /// Request the next element in a particular field
    Next { summarize: bool },
    /// Request the last element in a particular field
    Last { summarize: bool },
    /// Request the first element in a particular field
    First { summarize: bool },
    /// Request the entire contents of the specified field
    Get { summarize: bool },
    /// Does exactly what you think it does
    Post,
}
