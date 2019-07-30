use serde::ser::{Serialize, SerializeStruct, Serializer}; // Import serde

/// A log emitted during contract execution.
pub struct Log<'a> {
    /// The topic of the log
    topics: &'a [&'a str],
    /// The message of the log
    message: &'a [u8],
}

/// Implement a set of serialization helper methods.
impl<'a> Log<'a> {
    /// Serialize the log to a byte slice.
    pub fn serialize<S>(&self, serializer: S) -> Result<S:: Ok, S:: Error> where S: Serializer, {
        let mut state = serializer.serialize_struct("Log", 2)?; // Serialize log

        state.serialize_field("topics", &self.topics); // Serialize topics
        state.serialize_field("message", &self.message); // Serialize message

        return state.end(); // Return result
    }
}