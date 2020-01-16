use serde::Deserialize;
use std::{default, sync::Mutex};

/// A representation of the current state of some arbitrary query.
pub struct Ctx<'a> {
    /// Whether or not the query has been attempted yet
    attempted: bool,

    /// The response from the network as a byte slice
    raw_response: Mutex<Option<&'a [u8]>>,
}

impl<'a> Ctx<'a> {
    /// Initializes a new Ctx.
    pub fn new() -> Self {
        Self::default() // Just return the default version
    }

    /// Gets the body of the response in the query context.
    pub fn response<TResponse: Deserialize<'a>>(&self) -> Option<TResponse> {
        // If the context hasn't even been attempted, return None
        if !self.attempted {
            return None;
        }

        // Wait until the response has completed. If the response hasn't completed successfully, return
        // nothing.
        match self.raw_response.lock() {
            Ok(r) => {
                // Dereference the lock on the response
                let possibly_empty_resp_bytes = *r;

                // See if we can get the actual bytes in the response
                if let Some(non_empty_resp_bytes) = possibly_empty_resp_bytes {
                    // Deserialize the bytes and return them
                    bincode::deserialize::<TResponse>(non_empty_resp_bytes).ok()
                } else {
                    // There wasn't a response, so return nothing
                    None
                }
            }

            // An error occurred
            Err(e) => None,
        }
    }

    /// Flushes the context buffer.
    pub fn flush(&mut self) {
        // Reset the contents of the response buffer either way
        match self.raw_response.get_mut() {
            Ok(m) => *m = None,
            Err(_) => self.raw_response = Mutex::new(None),
        }

        self.attempted = false;
    }
}

impl<'a> default::Default for Ctx<'a> {
    /// Initializes a default version of the Ctx struct.
    fn default() -> Self {
        // Return an instance of the context struct with just a default status & response
        Self {
            // The response can't have come in yet, since nothing has actually happened
            attempted: false,
            raw_response: Mutex::new(None),
        }
    }
}
