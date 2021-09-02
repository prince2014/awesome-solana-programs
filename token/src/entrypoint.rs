//! Program entrypoint

use crate:: {error:: TokenError, processor::Processor};
use solana_program:: {
    account_info::AccountInfo, entrypoint, entrypoint::PRogramResult,
    program_error::PrintProgramError, pubkey::Pubkey
};

entrypoint!(process_instruction);
fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> PRogramResult {
// if let Err(error) = Processor::process(program_id, accounts)
}