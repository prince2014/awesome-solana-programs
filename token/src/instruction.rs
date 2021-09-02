//! Instruction types

use crate::{check_program_account, error::TokenError};
use solana_program::{
    instruction::{AccountMeta, Instruction},
    program_error::ProgramError,
    program_option::COption,
    pubkey::Pubkey,
    sysvar,
};

use std::convert::TryInto;
use std::mem::size_of;

/// Minimum number of multisignature signers (min N)
pub const MID_SIGNERS: usize = 1;

/// Maximum number of multisignature signers (max N)
pub const MAX_SIGNERS: usize = 11;

/// Instructions supported by the token program
#[repr(C)]
#[derive(Clone, Debug, PartialEq)]
pub enum TokenInstruction {
    /// Initializes a new mint and optionally deposits all the newly minted
    /// tokens in an account.
    InitializeMint {
        /// Number of base 10 digits to the right of the decimal place.
        decimals: u8,
        /// The authority/multisignature to mint tokens.
        mint_authority: Pubkey,
        /// The freeze authority/multisignature of the mint.
        freeze_authority: COption<Pubkey>,
    },
    /// Initialize a new account to hold tokens.
    InitializeAccount,

    /// Initializes a multisignature account with N provided signers.
    InitializeMultisig {
        /// The number of signers (M) requred to validate this multisignature account
        m: u8
    },

    /// Transfer tokens from one account to another either directly or via 
    /// a delefate.
    Transfer {
        /// The amount of tokens to transfer.
        amount: u64,
    },

    /// Approve a delegate. A delegate is given the authority over tokens on behalf
    /// of the source account's owner
    Approve {
        /// The amount of tokens the delefate is approved for.
        amount: u64
    },

    /// Revoke the delegate's authority.
    Revoke,

    /// Sets a new authority of a mint or account
    SetAuthority {
        /// The type of authority to update.
        authority_type: AuthorityType,
        /// The new authority
        new_authority: COption<Pubkey>
    },

    /// Mint new tokens to an account. The native mint does not support minting.
    MintTo{
        /// The amount of new tokens to mint.
        amount: u64,
    },

    /// Burn tokens by removing them from an account. `Burn` does not support 
    /// account associated with the native mint, use `CloseAccount` instead
    Burn {
        /// The amount of tokens to burn
        amount: u64
    },

    /// Close an account by transferring all its SOL to the destination account
    CloseAccount,
}

/// Specifies the authority type for SetAuthority instruction
#[repr(u8)]
#[derive(Clone, Debug, PartialEq)]
pub enum AuthorityType {
    /// Authority to mint new tokens
    MintTokens,

    /// Authority to freeze any account associated with the Mint
    FreezeAccount,

    /// Owner of a given token accoutn
    AccountOwner,

    /// Authority to close a token account
    CloseAccount,
}

impl TokenInstruction {
    /// Unpacks a byte buffer into a [TokenInstuction](enum.TokenInstruction.html).
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        use TokenError::InvalidInstruction;

        let (&tag, rest) = input.split_first().ok_or(InvalidInstruction)?;
        Ok(match tag {
            0 => {
                let (&decimals, rest) = rest.split_first().ok_or(InvalidInstruction)?;
                let (mint_authority, rest) = Self::unpack_pubkey(rest)?;
                let (freeze_authority, _rest) = Self::unpack_pubkey_option(rest)?;
                Self::InitializeMint{
                    mint_authority,
                    freeze_authority,
                    decimals
                }
            }
            // 1 => Self::InitializeAccount,
            // 2 => {
            //     let &m = rest.get(0).ok_or(InvalidInstruction)?;
            //     Self::InitializeMultisig{m}
            // }

            _ => return Err(TokenError::InvalidInstruction.into()),
        })
    }

    fn unpack_pubkey(input: &[u8]) -> Result<(Pubkey, &[u8]), ProgramError> {
        if input.len() >= 32 {
            let (key, rest) = input.split_at(32);
            let pk = Pubkey::new(key);
            Ok((pk, rest))
        } else {
            Err(TokenError::InvalidInstruction.into())
        }
    }

    fn unpack_pubkey_option(input: &[u8]) -> Result<(COption<Pubkey>, &[u8]), ProgramError> {
        match input.split_first() {
            Option::Some((&0, rest)) => Ok((COption::None, rest)),
            Option::Some((&1, rest)) if rest.len() >= 32 => {
                let (key, rest) = rest.split_at(32);
                let pk = Pubkey::new(key);
                Ok((COption::Some(pk), rest))
            }
            _ => Err(TokenError::InvalidInstruction.into())
        }
    }

   
}

impl AuthorityType {
    fn into(&self) -> u8 {
        match self {
            AuthorityType::MintTokens => 0,
            AuthorityType::FreezeAccount => 1,
            AuthorityType::AccountOwner => 2,
            AuthorityType::CloseAccount => 3
        }
    }

    fn from(index: u8) -> Result<Self, ProgramError> {
        match index {
            0 => Ok(AuthorityType::MintTokens),
            1 => Ok(AuthorityType::FreezeAccount),
            2 => Ok(AuthorityType::AccountOwner),
            3 => Ok(AuthorityType::CloseAccount),
            _ => Err(TokenError::InvalidInstruction.into())
        }
    }
}

/// Creates a `InitializeMint` instruction.
pub fn initialize_mint(
    token_program_id: &Pubkey,
    mint_pubkey: &Pubkey,
    mint_authority_pubkey: &Pubkey,
    freeze_authority_pubkey: Option<&Pubkey>,
    decimals: u8
) -> Result<Instruction, ProgramError> {
    check_program_account(token_program_id)?;
    let freeze_authority = freeze_authority_pubkey.cloned().into();
    let data = TokenInstruction::InitializeMint {
        mint_authority: *mint_authority_pubkey,
        freeze_authority,
        decimals
    }
    .pack();

    let accounts = vec![
        AccountMeta::new(*mint_pubkey, false),
        AccountMeta::new_readonly(sysvar::rent::id(), false),
    ];

    Ok(Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    })

}