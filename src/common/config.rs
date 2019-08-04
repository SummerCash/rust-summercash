use num::bigint::BigUint; // Add support for large unsigned integers

/// A container specifying a set of SummerCash protocol constants.
pub struct Config {
    /// The amount of finks per gas to give as a reward for validating a tx (i.e. increase rewards across the board) TODO: Gas table
    pub reward_per_gas: BigUint,
}
