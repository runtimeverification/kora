use super::common::BuildableInstruction;
use arbitrary::{Arbitrary, Error, Result, Unstructured};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, system_instruction::transfer};

#[derive(Arbitrary, Clone, Debug)]
pub enum FuzzSystemInstruction {
    Transfer,
}

impl BuildableInstruction for FuzzSystemInstruction {
    fn build(
        &self,
        u: &mut Unstructured,
        token: &Pubkey,
        accounts: &[Pubkey],
        atas: &[Pubkey],
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
