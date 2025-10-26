use super::common::BuildableInstruction;
use arbitrary::{Arbitrary, Error, Result, Unstructured};
use solana_sdk::{instruction::Instruction, program_error::ProgramError, pubkey::Pubkey};
use spl_token::{
    id,
    instruction::{approve, burn, transfer, AuthorityType, TokenInstruction},
};

#[derive(Arbitrary, Clone, Debug)]
pub enum FuzzSPLInstruction {
    Transfer,
    Approve,
    Burn,
}

impl BuildableInstruction for FuzzSPLInstruction {
    fn build(
        &self,
        u: &mut Unstructured,
        token: &Pubkey,
        accounts: &[Pubkey],
        atas: &[Pubkey],
    ) -> Result<Instruction> {
        match self {
            Self::Transfer => {
                let source_index = u.int_in_range(1..=accounts.len())? - 1;
                let source = atas[source_index];
                let authority = accounts[source_index];
                let destination = u.choose(atas)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                transfer(&id(), &source, &destination, &authority, &[], amount)
            }
            Self::Approve => {
                let source_index = u.int_in_range(1..=accounts.len())? - 1;
                let source = atas[source_index];
                let authority = accounts[source_index];
                let delegate = u.choose(atas)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                approve(&id(), &source, delegate, &authority, &[], amount)
            }
            Self::Burn => {
                let source_index = u.int_in_range(1..=accounts.len())? - 1;
                let account = atas[source_index];
                let authority = accounts[source_index];
                let amount = u.int_in_range(0..=1_000_000)?;
                burn(&id(), &account, token, &authority, &[], amount)
            }
        }
        .map_err(|_| Error::IncorrectFormat)
    }
}
