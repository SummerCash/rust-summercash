use clap::Clap;

#[derive(Clap, Clone)]
pub enum Create {
    /// Creates a new account.
    Account,

    /// Creates a new transaction.
    Transaction(Transaction),
}

#[derive(Clap, Clone)]
pub enum Get {
    /// Gets a particular account with the given address.
    Account(Account),

    /// Gets the balance of a particular account.
    Balance(Account),

    /// Gets a list of nodes contained in the working dag.
    Dag(UnitObject),

    /// Gets a list of transactions contained in the transaction cache.
    TransactionMemory(UnitObject),
}

#[derive(Clap, Clone)]
pub enum Lock {
    /// Locks a particular account with the given address.
    Account(CryptoAccount),
}

#[derive(Clap, Clone)]
pub enum Unlock {
    /// Unlocks a particular account with the given address.
    Account(CryptoAccount),
}

#[derive(Clap, Clone)]
pub enum Delete {
    /// Deletes an account with the given address.
    Account(Account),
}

#[derive(Clap, Clone)]
pub enum List {
    /// Gets a list of accounts stored on the disk.
    Accounts(UnitObject),

    /// Gets a list of transactions stored in the working DAG.
    Transactions(UnitObject),

    /// Gets a list of pending proposals held in the working runtime.
    Proposals(UnitObject),
}

#[derive(Clap, Clone)]
pub enum Sign {
    /// Signs the provided transaction with a given account
    Transaction(HashableObject),
}

#[derive(Clap, Clone)]
pub enum Publish {
    Transaction(HashableObject),
}

#[derive(Clap, Clone)]
pub struct Account {
    /// The address of the account
    pub address: String,
}

#[derive(Clap, Clone)]
pub struct CryptoAccount {
    /// The address of the account
    pub address: String,

    /// The encryption / decryption key used to unlock or lock the account
    pub key: String,
}

#[derive(Clap, Clone)]
pub struct UnitObject {}

#[derive(Clap, Clone)]
pub struct Transaction {
    /// A hex-encoded string representing the address of the sender of the transaction
    pub sender: String,

    /// A hex-encoded string representing the address of the recipient of the transaction
    pub recipient: String,

    /// The number of finks sent through the transaction
    pub amount: u128,

    /// A UTF-8-encoded payload sent along with the transaction
    pub payload: String,
}

#[derive(Clap, Clone)]
pub struct HashableObject {
    /// A hex-encoded string representing the hash of the object
    pub hash: String,
}
