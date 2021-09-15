#![deny(missing_docs)]
#![cfg_attr(not(test), forbid(unsafe_code)) ]

//! An ERC20-like Token program for Solana blockchain

pub mod error;
pub mod instruction;
pub mod processor;
pub mod state;

// #[cfg(not(feature = "no-entrypoint"))]
// mod entrypoint;

// Export current sdk types for downstream users building with a different sdk version
pub use solana_program;
use solana_program::{entrypoint::ProgramResult, program_error::ProgramError, pubkey::Pubkey};

solana_program::declare_id!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

/// Checks that the supplied program ID is the correct one for SPL-token
pub fn check_program_account(spl_token_program_id: &Pubkey) -> ProgramResult {
    if spl_token_program_id != &id() {
        return Err(ProgramError::IncorrectProgramId)
    }
    Ok(())
}
