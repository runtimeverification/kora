use crate::utils::common::TokenMetadata;

use super::common::BuildableInstruction;
use arbitrary::{Arbitrary, Result, Unstructured};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_instruction::transfer};

#[derive(Arbitrary, Clone, Debug)]
pub enum FuzzSystemInstruction {
    Transfer,
}

impl BuildableInstruction for FuzzSystemInstruction {
    fn build(
        &self,
        u: &mut Unstructured,
        _spl_meta: TokenMetadata,
        _spl_2022_meta: TokenMetadata,
        accounts: &[Pubkey],
    ) -> Result<Instruction> {
        let instruction = match self {
            Self::Transfer => {
                let source = u.choose(accounts)?;
                let destination = u.choose(accounts)?;
                let amount = u.int_in_range(0..=1_000_000)?;
                transfer(source, destination, amount)
            }
        };
        Ok(instruction)
    }
}
