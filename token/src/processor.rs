//! Program state processor

use std::{borrow::{Borrow, BorrowMut}, cmp::min};

use crate::{
    error::TokenError,
    instruction::{AuthorityType, TokenInstruction, MAX_SIGNERS},
    state::{Account, Mint, Multisig},
};

use num_traits::FromPrimitive;
use solana_program::{account_info::{next_account_info, AccountInfo}, decode_error::DecodeError, entrypoint::ProgramResult, entrypoint_deprecated::ProgramResult, msg, program_error::{PrintProgramError, ProgramError}, program_option::COption, program_pack::{IsInitialized, Pack}, pubkey::{self, Pubkey}, sysvar::{rent::Rent, Sysvar}};
use solana_sdk::account::accounts_equal;

/// Program state handler
pub struct Processor {}
impl Processor {
    fn _process_initialize_mint(
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
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: COption<Pubkey>,
    ) -> ProgramResult {
        Self::_process_initialize_mint(accounts, decimals, mint_authority, freeze_authority, true)
    }

    /// Processes an [InitializeMint2](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_mint2(
        accounts: &[AccountInfo],
        decimals: u8,
        mint_authority: Pubkey,
        freeze_authority: COption<Pubkey>,
    ) -> ProgramResult {
        Self::_process_initialize_mint(accounts, decimals, mint_authority, freeze_authority, false)
    }

    fn _process_initialize_account(
        accounts: &[AccountInfo],
        owner: Option<&Pubkey>,
        rent_sysvar_account: bool,
    ) -> ProgramResult {
        Ok(())
    }

    /// Processes an [InitializeAccount](enum.TokenInstruction.htmml) instruction.
    pub fn process_initialize_account(accounts: &[AccountInfo]) -> ProgramResult {
        Self::_process_initialize_account(accounts, None, true)
    }

    /// Processes an [InitializeAccount2](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_account2(accounts: &[AccountInfo], owner: Pubkey) -> ProgramResult {
        Self::_process_initialize_account(accounts, Some(&owner), true)
    }

    /// Processes an [InitializeAccount3](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_account3(accounts: &[AccountInfo], owner: Pubkey) -> ProgramResult {
        Self::_process_initialize_account(accounts, Some(&owner), false)
    }

    fn _process_initialize_multisig(
        accounts: &[AccountInfo],
        m: u8,
        rent_sysvar_account: bool,
    ) -> ProgramResult {
        Ok(())
    }

    /// Processes a [InitializeMultisig](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_multisig(accounts: &[AccountInfo], m: u8) -> ProgramResult {
        Self::_process_initialize_multisig(accounts, m, true)
    }

    /// Processes a [InitializeMultisig2](enum.TokenInstruction.html) instruction.
    pub fn process_initialize_multisig2(accounts: &[AccountInfo], m: u8) -> ProgramResult {
        Self::_process_initialize_multisig(accounts, m, false)
    }

    /// Processes a [Transfer](enum.TokenInstruction.html) instruction.
    pub fn process_transfer(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        expected_decimals: Option<u8>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let source_account_info = next_account_info(account_info_iter)?;

        let expected_mint_info = if let Some(expected_decimals) = expected_decimals {
            Some((next_account_info(account_info_iter)?, expected_decimals))
        } else {
            None
        };

        let dest_account_info = next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;

        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        let mut dest_account = Account::unpack(&dest_account_info.data.borrow())?;

        if source_account.is_frozen() || dest_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }

        if source_account.mint != dest_account.mint {
            return Err(TokenError::MintMismatch.into());
        }

        if let Some((mint_info, expected_decimals)) = expected_mint_info {
            if source_account.mint != *mint_info.key {
                return Err(TokenError::MintMismatch.into());
            }

            let mint = Mint::unpack(&mint_info.data.borrow_mut())?;
            if expected_decimals != mint.decimals {
                return Err(TokenError::MintDecimalsMismatch.into());
            }
        }

        let self_transfer = source_account_info.key == dest_account_info.key;

        match source_account.delegate {
            COption::Some(ref delegate) if authority_info.key == delegate => {
                Self::validate_owner(
                    program_id,
                    delegate,
                    authority_info,
                    account_info_iter.as_slice(),
                )?;
                if source_account.delegated_amount < amount {
                    return Err(TokenError::InsufficientFunds.into());
                }
                if !self_transfer {
                    source_account.delegated_amount = source_account
                        .delegated_amount
                        .checked_sub(amount)
                        .ok_or(TokenError::Overflow)?;
                    if source_account.delegated_amount == 0 {
                        source_account.delegate = COption::None;
                    }
                }
            }
            _ => Self::validate_owner(
                program_id,
                &source_account.owner,
                authority_info,
                account_info_iter.as_slice(),
            )?,
        };

        if self_transfer {
            return Ok(());
        }

        source_account.amount = source_account
            .amount
            .checked_sub(amount)
            .ok_or(TokenError::Overflow)?;
        dest_account.amount = dest_account
            .amount
            .checked_add(amount)
            .ok_or(TokenError::Overflow)?;

        if source_account.is_native() {
            let source_starting_lamports = source_account_info.lamports();
            **source_account_info.lamports.borrow_mut() = source_starting_lamports
                .checked_sub(amount)
                .ok_or(TokenError::Overflow)?;

            let dest_starting_lamports = dest_account_info.lamports();
            **dest_account_info.lamports.borrow_mut() = dest_starting_lamports
                .checked_add(amount)
                .ok_or(TokenError::Overflow)?;
        }

        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;
        Account::pack(dest_account, &mut dest_account_info.data.borrow_mut())?;

        Ok(())
    }

    /// Process an [Approve](enum.TokenInstruction.html) instruction.
    pub fn process_approve(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        expected_decimals: Option<u8>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let source_account_info = next_account_info(account_info_iter)?;

        let expected_mint_info = if let Some(expected_decimals) = expected_decimals {
            Some((next_account_info(account_info_iter)?, expected_decimals))
        } else {
            None
        };

        let delegate_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;

        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        if source_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }

        if let Some((mint_info, expected_decimals)) = expected_mint_info {
            if source_account.mint != mint_info.key {
                return Err(TokenError::MintMismatch.into());
            }

            let mint = Mint::unpack(&mint_info.data.borrow_mut())?
            if expected_decimals != *mint_info.key{
                return Err(TokenError::MintDecimalsMismatch.into());
            }
        }

        Self::validate_owner(program_id, &source_account_info, owner_info, account_info_iter.as_slice())?;

        source_account_info.delegate = COption::Some(*delegate_info.key);
        source_account.delegated_amount = amount;

        Account::pack(source_account, &mut source_account_info.borrow_mut());
        Ok(())
    }

    /// Processes an [Approve](enum.TokenInstruction.html) instruction.
    pub fn process_revoke(program_id: &Pubkey, accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let source_account_info = next_account_info(account_info_iter)?;
        
        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        
        let owner_info = next_account_info(account_info_iter)?;

        if source_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }

        Self::validate_owner(program_id, &source_account.owner, owner_info, account_info_iter.as_slice())?;

        source_account.delegate = COption::None;
        source_account.delegated_amount = 0;

        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;

        Ok(()) 
  }

  pub fn process_set_authority(
      program_id: &Pubkey,
      accounts: &[AccountInfo],
      authority_type: AuthorityType,
      new_authority: COption<Pubkey>,
  ) -> ProgramResult {
      let account_info_iter = &mut accounts.iter();
      let account_info = next_account_info(account_info_iter)?;
      let authority_info = next_account_info(account_info_iter)?;

      if account_info.data_len() == Account::get_packed_len() {
        let mut account = Account::unpack(&account_info.data.borrow())?;
        
        if account.is_frozen() {
              return Err(TokenError::AccountFrozen.into());
          }

        match authority_type {
            AuthorityType::AccountOwner => {
                Self::validate_owner(
                    program_id, &account.owner, authority_info, account_info_iter.as_slice(),
                )?;
                if let COption::Some(authority) = new_authority {
                    account.owner = authority;
                } else {
                    return Err(TokenError::InvalidInstruction.into());
                }

                account.delegate = COption::None;
                account.delegated_amount = 0;

                if account.is_native() {
                    account.close_authority = COption::None;
                }
            }
            AuthorityType::CloseAccount => {
                let authority = account.close_authority.unwrap_or(account.owner);
                Self::validate_owner(
                    program_id, 
                    &authority, authority_info, account_info_iter.as_slice(),
                )?;
                account.close_authority = new_authority;
            }

            _ =>{
                return  Err(TokenError::AuthorityTypeNotSupported.into());
            }
        }
        Account::pack(account, &mut account_info.data.borrow_mut())?;
    } else if account_info.data_len() == Mint::get_packed_len() {
          let mut mint = Mint::unpack(&account_info.data.borrow())?;
          match authority_type {
              AuthorityType::MintTokens => {
                let mint_authority = mint
                    .mint_authority
                    .ok_or(Into::<ProgramError>::into(TokenError::FixedSupply))?;
                Self::validate_owner(program_id, &mint_authority, authority_info, account_info_iter.as_slice(),)?;
                mint.mint_authority = new_authority;
              }
              AuthorityType::FreezeAccount => {
                  let freeze_authority = mint
                  .freeze_authority
                  .ok_or(Into::<ProgramError>::into(TokenError::MintCannotFreeze))?;
                  Self::validate_owner(program_id, &freeze_authority, authority_info,
                     account_info_iter.as_slice(),)?;
                    mint.freeze_authority = new_authority;
              }

              _ => {
                  return Err(TokenError::AuthorityTypeNotSupported.into());
              }
          }
          Mint::pack(mint, &mut account_info.data.borrow_mut())?;
      } else {
          return  Err(ProgramError::InvalidArgument);
      }

    Ok(())
  }

    /// Processes a [MintTo](enum.TokenInstruction.html) instruction.
    pub fn process_mint_to(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        amount: u64,
        expected_decimals: Option<u8>,
    ) ->ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let mint_info = next_account_info(account_info_iter)?;
        let dest_account_info = next_account_info(account_info_iter)?;
        let owner_info = next_account_info(account_info_iter)?;

        let mut dest_account = Account::unpack(&dest_account_info.data.borrow())?;
        if dest_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        if dest_account.is_native() {
            return Err(TokenError::NativeNotSupported.into());
        }
        if mint_info.key != &dest_account.mint {
            return Err(TokenError::MintMismatch.into());
        }

        let mut mint = Mint::unpack(&mint_info.data.borrow())?;
        if let Some(expected_decimals) = expected_decimals {
            if expected_decimals != mint.decimals {
                return Err(TokenError::MintDecimalsMismatch.into());
            }
        }
        match mint.mint_authority {
            COption::Some(mint_authority) => Self::validate_owner(
                program_id,
                 &mint_authority, 
                 owner_info, 
                 account_info_iter.as_slice(),
            )?,
            COption::None => return  Err(TokenError::FixedSupply.into()),
        }

        dest_account.amount = dest_account
        .amount
        .checked_add(amount)
        .ok_or(TokenError::Overflow)?;

        mint.supply = mint
        .supply
        .checked_add(amount)
        .ok_or(TokenError::Overflow)?;

        Account::pack(dest_account, &mut dest_account_info.data.borrow_mut())?;
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;

        Ok(())
    }

    /// Processes a [Burn](enum.TokenInstruction.html) instruction.
    pub fn procee_burn(
        program_id: &Pubkey,
        accounts:&[AccountInfo],
        amount: u64,
        expected_decimals: Option<u8>,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        let source_account_info = next_account_info(account_info_iter)?;
        let mint_info  =next_account_info(account_info_iter)?;
        let authority_info = next_account_info(account_info_iter)?;
        
        let mut source_account = Account::unpack(&source_account_info.data.borrow())?;
        let mut mint = Mint::unpack(&mint_info.data.borrow())?;

        if source_account.is_frozen() {
            return Err(TokenError::AccountFrozen.into());
        }
        if source_account.is_native() {
            return Err(TokenError::NativeNotSupported.into());
        }
        if source_account.amount < amount {
            return Err(TokenError::InsufficientFunds.into());
        }
        if mint_info.key != &source_account.mint {
            return  Err(TokenError::MintMismatch.into());
        }

        if let Some(expected_decimals) = expected_decimals {
            if  expected_decimals != mint.decimals {
                return Err(TokenError::MintDecimalsMismatch.into());
            }
        }

        match source_account.delegate {
            COption::Some(ref delegate) if authority_info.key == delegate => {
                Self::validate_owner(
                    program_id,
                     delegate,
                      authority_info,
                       account_info_iter.as_slice(),
                    )?;
            
                if source_account.delegated_amount < amount  {
                    return Err(TokenError::InsufficientFunds.into());
                }
          
                source_account.delegated_amount = source_account
                    .delegated_amount
                    .checked_sub(amount)
                    .ok_or(TokenError::Overflow)?;

                if source_account.delegated_amount == 0 {
                    source_account.delegate = COption::None;
                }
            } 
            _ => Self::validate_owner(
                program_id,
                &source_account.owner,
                authority_info,
                account_info_iter.as_slice(),
            )?,
        }

        source_account.amount = source_account
        .amount
        .checked_sub(amount)
        .ok_or(TokenError::Overflow)?;
        
        mint.supply = mint
         .supply
         .checked_sub(amount)
         .ok_or(TokenError::Overflow)?;

        Account::pack(source_account, &mut source_account_info.data.borrow_mut())?;
        Mint::pack(mint, &mut mint_info.data.borrow_mut())?;
        Ok(())
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
                Self::process_initialize_mint(accounts, decimals, mint_authority, freeze_authority)
            }

            TokenInstruction::InitializeMint2 {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                msg!("Instruction: InitializeMint2");
                Self::process_initialize_mint2(accounts, decimals, mint_authority, freeze_authority)
            }

            TokenInstruction::InitializeAccount => {
                msg!("Instruction: InitializeAccount");
                Self::process_initialize_account(accounts)
            }

            TokenInstruction::InitializeAccount2 { owner } => {
                msg!("Instruction: InitializeAccount2");
                Self::process_initialize_account2(accounts, owner)
            }

            TokenInstruction::InitializeMultisig { m } => {
                msg!("Instruction: InintializeMultisig");
                Self::process_initialize_multisig(accounts, m)
            }

            TokenInstruction::InitializeMultisig2 { m } => {
                msg!("Instruction: InintializeMultisig");
                Self::process_initialize_multisig(accounts, m)
            }
            TokenInstruction::Transfer { amount } => todo!(),
            TokenInstruction::Approve { amount } => todo!(),
            TokenInstruction::Revoke => todo!(),
            TokenInstruction::SetAuthority {
                authority_type,
                new_authority,
            } => todo!(),
            TokenInstruction::MintTo { amount } => todo!(),
            TokenInstruction::Burn { amount } => todo!(),
            TokenInstruction::CloseAccount => todo!(),
            // TokenInstruction::Transfer {amount} => {
            //     msg!("Instruction: Transfer"):
            //     Self::process_transfer()
            // }
        }
    }

    /// Validates owner(s) are present
    pub fn validate_owner(
        program_id: &Pubkey,
        expectted_owner: &Pubkey,
        owner_account_info: &AccountInfo,
        signers: &[AccountInfo],
    ) -> ProgramResult {
        if expectted_owner != owner_account_info.key {
            return Err(TokenError::OwnerMismatch.into());
        }
        if program_id == owner_account_info.owner
            && owner_account_info.data_len() == Multisig::get_packed_len()
        {
            let multisig = Multisig::unpack(&owner_account_info.data.borrow())?;
            let mut num_signers = 0;
            let mut matched = [false; MAX_SIGNERS];
            for signer in signers.iter() {
                for (position, key) in multisig.signers[0..multisig.n as usize].iter().enumerate() {
                    if key == signer.key && !matched[position] {
                        if !signer.is_signer {
                            return Err(ProgramError::MissingRequiredSignature);
                        }
                        matched[position] = true;
                        num_signers += 1;
                    }
                }
            }
            if num_signers < multisig.m {
                return Err(ProgramError::MissingRequiredSignature);
            }
            return Ok(());
        } else if !owner_account_info.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        Ok(())
    }
}
