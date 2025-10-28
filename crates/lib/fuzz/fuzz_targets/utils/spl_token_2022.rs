use super::common::{BuildableInstruction, TokenMetadata};
use arbitrary::{Arbitrary, Error, Result, Unstructured};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_token_2022::instruction::{approve, burn, transfer};

#[derive(Arbitrary, Clone, Debug)]
pub enum FuzzSPL2022Instruction {
    Transfer,
    Approve,
    Burn,
}

impl BuildableInstruction for FuzzSPL2022Instruction {
    fn build(
        &self,
        u: &mut Unstructured,
        _spl_meta: TokenMetadata,
        spl_2022_meta: TokenMetadata,
        accounts: &[Pubkey],
    ) -> Result<Instruction> {
        let TokenMetadata { program, mint_pubkey, decimals, atas, .. } = spl_2022_meta;
        match self {
            Self::Transfer => {
                let source_index = u.int_in_range(1..=accounts.len())? - 1;
                let source = atas[source_index];
                let authority = accounts[source_index];
                let destination = u.choose(&atas)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                transfer(&program, &source, destination, &authority, &[], amount)
            }
            Self::Approve => {
                let source_index = u.int_in_range(1..=accounts.len())? - 1;
                let source = atas[source_index];
                let owner = accounts[source_index];
                let delegate = u.choose(&atas)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                approve(&program, &source, delegate, &owner, &[], amount)
            }
            Self::Burn => {
                let source_index = u.int_in_range(1..=accounts.len())? - 1;
                let account = atas[source_index];
                let owner = accounts[source_index];
                let amount = u.int_in_range(0..=1_000_000)?;
                burn(&program, &account, &mint_pubkey, &owner, &[], amount)
            }
        }
        .map_err(|_| Error::IncorrectFormat)
    }
}
