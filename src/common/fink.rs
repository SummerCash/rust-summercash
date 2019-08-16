use num::bigint::{BigInt, BigUint, Sign}; // Add support for large unsigned integers
use num::rational::{BigRational, Ratio}; // Add support for large floats

use std::str::FromStr; // Let the bigint library implement from_str

/* BEGIN EXPORTED METHODS */

/// Get the number of finks per SMC.
///
/// # Example
///
/// ```
/// use summercash::common::fink; // Import the fink unit conversion utility
///
/// let n_finks_per = fink::num_finks_per_smc(); // 1000000000000000000
/// ```
pub fn num_finks_per_smc() -> BigUint {
    BigUint::from_str("1000000000000000000").unwrap() // Return number of finks per SMC
}

/// Converts a given quantity of SummerCash denoted in Finks to SMC.
///
/// # Arguments
///
/// * `n_finks` - The number of finks to convert to SMC
///
/// # Example
///
/// ```
/// use summercash::common::fink; // Import the fink -> SMC conversion utility
/// use num::bigint::{BigInt, BigUint, Sign}; // Add support for large unsigned integers
///
/// use std::str::FromStr; // Let the bigint library implement from_str
///
/// let n_smc = fink::convert_finks_to_smc(BigUint::from_str("1000000000000000000").unwrap()); // 1 SMC
/// ```
pub fn convert_finks_to_smc(n_finks: BigUint) -> BigRational {
    Ratio::from_integer(BigInt::from_biguint(Sign::Plus, n_finks))
        / Ratio::from_integer(BigInt::from_biguint(Sign::Plus, num_finks_per_smc())) // Return number of SMCs
}

/// Converts a given quantity of SummerCash denoted in SMC to Finks.
///
/// # Arguments
///
/// * `n_smc` - The number of SMC to convert to Finks
///
/// # Example
///
/// ```
/// use summercash::common::fink; // Import the fink unit conversion utility
/// use num::rational::{BigRational, Ratio}; // Add support for large floats\
///
/// use std::str::FromStr; // Let the bigint library implement from_str
///
/// let n_finks = fink::convert_smc_to_finks(BigRational::from_str("1/1").unwrap()); // 1000000000000000000 finks
/// ```
pub fn convert_smc_to_finks(n_smc: BigRational) -> BigUint {
    (n_smc * Ratio::from_integer(BigInt::from_biguint(Sign::Plus, num_finks_per_smc())))
        .to_integer()
        .to_biguint()
        .unwrap() // Return number of finks
}

/* END EXPORTED METHODS */

// Unit tests
#[cfg(test)]
mod tests {
    use super::*; // Import names from outside module

    #[test]
    fn test_num_finks_per_smc() {
        assert_eq!(
            num_finks_per_smc(),
            BigUint::from_str("1000000000000000000").unwrap()
        ); // Should be 1000000000000000000 (the only reason we're doing this is to check for potential panics)
    }

    #[test]
    fn test_convert_finks_to_smc() {
        assert_eq!(
            convert_finks_to_smc(num_finks_per_smc()),
            BigRational::from_str("1/1").unwrap()
        ); // Should be 1 SMC
    }

    #[test]
    fn test_convert_smc_to_finks() {
        assert_eq!(
            convert_smc_to_finks(BigRational::from_str("1/1").unwrap()),
            num_finks_per_smc()
        ); // Should be 1000000000000000000
    }
}
