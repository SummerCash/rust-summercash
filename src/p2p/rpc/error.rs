/// An error code representing a signature value that was unable to be derived.
pub const ERROR_SIGNATURE_UNDEFINED: i64 = 0;

/// An error code representing the inability of the executor to open an account with the given address.
/// This error is returned if the user attempts to open an account that is locked.
pub const ERROR_UNABLE_TO_OPEN_ACCOUNT: i64 = 1;

/// An error code representing the inability of the executor to write an account to persistent memory.
pub const ERROR_UNABLE_TO_WRITE_ACCOUNT: i64 = 2;
