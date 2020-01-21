/// An error code representing a signature value that was unable to be derived.
pub const ERROR_SIGNATURE_UNDEFINED: i64 = 0;

/// An error code representing the inability of the executor to open an account with the given address.
/// This error is returned if the user attempts to open an account that is locked.
pub const ERROR_UNABLE_TO_OPEN_ACCOUNT: i64 = 1;

/// An error code representing the inability of the executor to write an account to persistent memory.
pub const ERROR_UNABLE_TO_WRITE_ACCOUNT: i64 = 2;

/// An error code representing the inability of the executor to read an account from a file.
pub const ERROR_UNABLE_TO_READ_ACCOUNT: i64 = 3;

/// An error code representing the inability of the executor to generate a random seed.
pub const ERROR_UNABLE_TO_GENERATE_RANODM: i64 = 4;

/// An error code representing a failure in the executor's encryption process.
pub const ERROR_ENCRYPTION_FAILED: i64 = 5;

/// An error code representing a failure in the executor's decryption process.
pub const ERROR_DECRYPTION_FAILED: i64 = 6;
