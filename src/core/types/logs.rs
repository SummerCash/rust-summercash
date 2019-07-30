/// A log emitted during contract execution.
pub struct Log<'a> {
    /// The topic of the log
    topic: &'a str,
    /// The message of the log
    message: &'a [u8],
}
