use serde::Deserialize;
use std::{default, sync::Mutex};

/// A representation of the current state of some arbitrary query.
pub struct Ctx<'a, TBody: Deserialize<'a>> {
    /// Whether or not the query has finished or not
    status: bool,

    /// Whether or not the query has been attempted yet
    attempted: bool,

    /// The response from the network as a byte slice
    raw_response: Option<&'a [u8]>,

    /// The response from the query
    response: Mutex<Option<TBody>>,
}

impl<'a, TBody: Deserialize<'a>> Ctx<'a, TBody> {
    /// Initializes a new Ctx.
    pub fn new() -> Self {
        Self::default() // Just return the default version
    }

    /// Gets the body of the response in the query context.
    pub fn response(&self) -> Option<TBody> {
        // If the context hasn't even been attempted, return None
        if !self.attempted {
            return None;
        }

        // Wait until the response has completed. If the response hasn't completed successfully, return
        // nothing.
        match self.response.lock() {
            Ok(r) => *r,
            Err(e) => None,
        }
    }
}

impl<'a, TBody: Deserialize<'a>> default::Default for Ctx<'a, TBody> {
    /// Initializes a default version of the Ctx struct.
    fn default() -> Self {
        // Return an instance of the context struct with just a default status & response
        Self {
            // The response can't have come in yet, since nothing has actually happened
            status: false,
            attempted: false,
            raw_response: None,
            response: Mutex::new(None),
        }
    }
}
