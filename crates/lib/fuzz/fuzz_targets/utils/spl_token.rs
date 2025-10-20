use super::common::FuzzPubkey;
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

impl FuzzSPLInstruction {
    pub fn build(
        &self,
        u: &mut Unstructured,
        token: &Pubkey,
        atas: &[&Pubkey],
    ) -> Result<Instruction> {
        match self {
            Self::Transfer => {
                let source = u.choose(atas)?;
                let destination = u.choose(atas)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                transfer(&id(), source, destination, token, &[], amount)
            }
            Self::Approve => {
                let source = u.choose(atas)?;
                let delegate = u.choose(atas)?;
                let owner = u.choose(atas)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                approve(&id(), source, delegate, owner, &[], amount)
            }
            Self::Burn => {
                let account = u.choose(atas)?;
                let authority = u.choose(atas)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                burn(&id(), account, token, authority, &[], amount)
            }
        }
        .map_err(|_| Error::IncorrectFormat)
    }
}