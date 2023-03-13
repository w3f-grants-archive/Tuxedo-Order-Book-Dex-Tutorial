//! A constraint checker is a piece of logic that determines whether a transaction as a whole is valid
//! and should be committed. Most tuxedo pieces will provide one or more constraint checkers.
//! Constraint Checkers do not typically calculate the correct final state, but rather determine whether the
//! proposed final state (as specified by the output set) meets the necessary constraints.

use sp_std::{fmt::Debug, vec::Vec};

use crate::{dynamic_typing::DynamicallyTypedData, types::Output, Verifier};
use parity_scale_codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::transaction_validity::TransactionPriority;

/// A single constraint checker that a transaction can choose to call. Checks whether the input
/// and output data from a transaction meets the codified constraints.
///
/// Additional transient information may be passed to the constraint checker by including it in the fields
/// of the constraint checker struct itself. Information passed in this way does not come from state, nor
/// is it stored in state.
pub trait SimpleConstraintChecker: Debug + Encode + Decode + Clone {
    /// The error type that this constraint checker may return
    type Error: Debug;

    /// The actual check validation logic
    fn check(
        &self,
        input_data: &[DynamicallyTypedData],
        output_data: &[DynamicallyTypedData],
    ) -> Result<TransactionPriority, Self::Error>;
}

pub trait ConstraintChecker: Debug + Encode + Decode + Clone {
    /// the error type that this constraint checker may return
    type Error: Debug;

    /// The actual check validation logic
    fn check<V: Verifier>(
        &self,
        inputs: &[Output<V>],
        outputs: &[Output<V>],
    ) -> Result<TransactionPriority, Self::Error>;
}

// This blanket implementation makes it so that any type that chooses to
// implement the Simple trait also implements the more Powerful trait. This way
// the executive can always just call the more Powerful trait.
impl<T: SimpleConstraintChecker> ConstraintChecker for T {
    // Use the same error type used in the simple implementation.
    type Error = <T as SimpleConstraintChecker>::Error;

    fn check<V: Verifier>(
        &self,
        inputs: &[Output<V>],
        outputs: &[Output<V>],
    ) -> Result<TransactionPriority, Self::Error> {
        // Extract the input data
        let input_data: Vec<DynamicallyTypedData> =
            inputs.iter().map(|o| o.payload.clone()).collect();

        // Extract the output data
        let output_data: Vec<DynamicallyTypedData> =
            outputs.iter().map(|o| o.payload.clone()).collect();

        // Call the simple constraint checker
        SimpleConstraintChecker::check(self, &input_data, &output_data)
    }
}

/// Utilities for writing constraint-checker-related unit tests
#[cfg(feature = "std")]
pub mod testing {
    use super::*;

    /// A testing checker that passes (with zero priority) or not depending on
    /// the boolean value enclosed.
    #[derive(Serialize, Deserialize, Encode, Decode, Debug, Clone, PartialEq, Eq)]
    pub struct TestConstraintChecker {
        /// Whether the checker should pass.
        pub checks: bool,
    }

    impl SimpleConstraintChecker for TestConstraintChecker {
        type Error = ();

        fn check(
            &self,
            _input_data: &[DynamicallyTypedData],
            _output_data: &[DynamicallyTypedData],
        ) -> Result<TransactionPriority, ()> {
            if self.checks {
                Ok(0)
            } else {
                Err(())
            }
        }
    }

    #[test]
    fn test_checker_passes() {
        let result = SimpleConstraintChecker::check(&TestConstraintChecker { checks: true }, &[], &[]);
        assert_eq!(result, Ok(0));
    }

    #[test]
    fn test_checker_fails() {
        let result = SimpleConstraintChecker::check(&TestConstraintChecker { checks: false }, &[], &[]);
        assert_eq!(result, Err(()));
    }
}
