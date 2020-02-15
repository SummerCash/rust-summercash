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

/// An error code representing the inability of the executor to delete an account.
pub const ERROR_UNABLE_TO_DELETE_ACCOUNT: i64 = 7;

/// An error code representing the inabiliity of the executor to obtain a reading lock.
pub const ERROR_UNABLE_TO_OBTAIN_LOCK: i64 = 8;

/// An error code representing the inability of the executor to get a lock on the last valid state entry.
pub const ERROR_UNABLE_TO_OBTAIN_STATE_REF: i64 = 9;

/// An error code representing the inability of the executor to find/open the genesis file for the DAG.
pub const ERROR_UNABLE_TO_OPEN_GENESIS_CONFIG: i64 = 10;

/// An error code representing the inability of the executor to create a genesis block.
pub const ERROR_UNABLE_TO_CREATE_GENESIS: i64 = 11;

/// An error code representing the inability of the executor to serialize some SummerCash object.
pub const ERROR_SERIALIZATION_FAILED: i64 = 12;

/// An error code representing the inability of the executor to open a SummerCash transaction.
pub const ERROR_UNABLE_TO_OPEN_TRANSACTION: i64 = 13;

/// An error code representing the inability of the executor to persist the SummerCash transaction to the disk.
pub const ERROR_UNABLE_TO_WRITE_TRANSACTION: i64 = 14;

/// An error code representing the inability of the executor to create a proposal from the given SummerCash object.
pub const ERROR_UNABLE_TO_CREATE_PROPOSAL: i64 = 15;

/// An error code representing the inability of the executor to read the DAG from the disk.
pub const ERROR_UNABLE_TO_READ_DAG: i64 = 16;
