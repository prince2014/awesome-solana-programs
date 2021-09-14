//! Program state processor

use crate::{
    error::TokenError,
    instruction::{AuthorityType, TokenInstruction, MAX_SIGNERS },
};

use num_traits::FromPrimitive;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    decode_error::DecodeError,
    entrypoint::ProgramResult,
    msg,
    program_error::{PrintProgramError, ProgramError},
    program_option::COption,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{rent::Rent, Sysvar},
};

/// Program state handler
pub struct Processor{}
impl Processor {
    fn _processe_initialize_mint(
        accounts: &[AccountInfo],
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: COption<Pubkey>,
        rent_sysvar_account: bool,
    ) -> ProgramResult {
       
        Ok(())
    }

    /// Processes an [InitializeMint](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_mint(
        accounts: &[AccountInfo],
        decimals:u8,
        mint_authority: Pubkey,
        freeze_authority: COption<Pubkey>,
    ) -> ProgramResult {
        Self::_processe_initialize_mint(accounts, decimals, mint_authority, freeze_authority, true)
    }

    /// Processes an [InitializeMint2](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_mint2(
        accounts: &[AccountInfo],
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: COption<Pubkey>,
    ) -> ProgramResult {
        Self::_processe_initialize_mint(accounts, decimals, mint_authority, freeze_authority, false)
    }

    fn _process_initialize_account(
        accounts: &[AccountInfo],
        owner: Option<&Pubkey>,
        rent_sysvar_account: bool,
    )-> ProgramResult{
        Ok(())
    }

    /// Processes an [InitializeAccount](enum.TokenInstruction.htmml) instruction.
    pub fn process_initialize_account(accounts: &[AccountInfo]) -> ProgramResult{
        Self::_process_initialize_account(accounts,None, true)
    }

    /// Processes an [InitializeAccount2](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_account2(
        accounts: &[AccountInfo],
        owner: Pubkey,
    ) -> ProgramResult {
        Self::_process_initialize_account(accounts, Some(&owner), true)
    }

    /// Processes an [InitializeAccount3](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_account3(
        accounts: &[AccountInfo],
        owner: Pubkey,
    )->ProgramResult{
        Self::_process_initialize_account(accounts, Some(&owner), false)
    }

}

/// Processes an [Instruction](enum.Instruction.html).
pub fn process(program_id: &Pubkey, accounts: &[AccountInfo], input: &[u8]) -> ProgramResult {
    let instruction = TokenInstruction::unpack(input)?;

    match instruction {
        TokenInstruction::InitializeMint {
            decimals,
            mint_authority,
            freeze_authority,
        } => {
            msg!("Instruction: InitializeMint");
            Self::process_initialize_mint(account, decimals, mint_authority, freeze_authority)
        }
    }
}