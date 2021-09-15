//! Error types

use num_derive::FromPrimitive;
use solana_program::{decode_error::DecodeError, program_error::ProgramError};
use thiserror::Error;

/// Errors that may be returned by the Token program.
#[derive(Clone, Debug, Eq, Error, FromPrimitive, PartialEq)]
pub enum TokenError {
    // 0
    /// Lamport balance below rent-exempt threshold.
    #[error("Lamport balance below rent-exempt threshold")]
    NotRentExempt,

    /// Insufficient funds for the operation requested.
    #[error("Insufficient funds")]
    InsufficientFunds,

    /// Invalid Mint.
    #[error("Invalid Mint")]
    InvalidMint,

    /// Account not associated with this Mint.
    #[error("Account not associated with this Mint")]
    MintMismatch,

    /// Owner does not match.
    #[error("Owner does not match")]
    OwnerMismatch,


    // 10
    /// Instruction does not support native tokens
    #[error("Instruction does not support native tokens")]
    NativeNotSupported,

    /// Non-native account can only be closed if its blanace is zero
    #[error("Non-native account can only be closed if its balance is zero")]
    NonNativeHashBalance,
    
    /// Invalid instruction
    #[error("Invalid instruction")]
    InvalidInstruction,

    /// Operation overflowed
    #[error("Operation overflowed")]
    Overflow,

    // 15
    /// Account does not support specified authority type.
    #[error("Account does not support specified authority type")]
    AuthorityTypeNotSupported,
    /// Account is frozen; all account operations will fail
    #[error("Account is frozen")]
    AccountFrozen,
    /// Mint decimals mismatch between the client and mint
    #[error("The provided decimals value different from the Mint decimals")]
    MintDecimalsMismatch,
    
}

impl From<TokenError> for ProgramError {
    fn from(e: TokenError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl<T> DecodeError<T> for TokenError {
    fn type_of() -> &'static str {
        "TokenError"
    }
}