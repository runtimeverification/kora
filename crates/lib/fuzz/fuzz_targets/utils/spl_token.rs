use super::common::{BuildableInstruction, TokenMetadata};
use arbitrary::{Arbitrary, Error, Result, Unstructured};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_token::instruction::{approve, burn, transfer};

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
        spl_meta: TokenMetadata,
        _spl_2022_meta: TokenMetadata,
        accounts: &[Pubkey],
    ) -> Result<Instruction> {
        let TokenMetadata { program, mint_pubkey, decimals, atas, .. } = spl_meta;
        match self {
            Self::Transfer => {
                let source_index = u.int_in_range(1..=accounts.len())? - 1;
                let source = atas[source_index];
                let authority = accounts[source_index];
                let destination = u.choose(&atas)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                transfer(&program, &source, &destination, &authority, &[], amount)
            }
            Self::Approve => {
                let source_index = u.int_in_range(1..=accounts.len())? - 1;
                let source = atas[source_index];
                let authority = accounts[source_index];
                let delegate = u.choose(&atas)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                approve(&program, &source, delegate, &authority, &[], amount)
            }
            Self::Burn => {
                let source_index = u.int_in_range(1..=accounts.len())? - 1;
                let account = atas[source_index];
                let authority = accounts[source_index];
                let amount = u.int_in_range(0..=1_000_000)?;
                burn(&program, &account, &mint_pubkey, &authority, &[], amount)
            }
        }
        .map_err(|_| Error::IncorrectFormat)
    }
}
