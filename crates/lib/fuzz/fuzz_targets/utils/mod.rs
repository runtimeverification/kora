pub mod common;
pub mod litesvm;
pub mod spl_token;
pub mod spl_token_2022;
pub mod system_interface;

pub use litesvm::LiteSVMSender;

use arbitrary::Arbitrary;

use crate::utils::common::TokenMetadata;

#[derive(Arbitrary, Clone, Debug)]
pub enum FuzzInstruction {
    System(system_interface::FuzzSystemInstruction),
    SPLToken(spl_token::FuzzSPLInstruction),
    SPLToken2022(spl_token_2022::FuzzSPL2022Instruction),
}

impl common::BuildableInstruction for FuzzInstruction {
    fn build(
        &self,
        u: &mut arbitrary::Unstructured,
        spl_meta: TokenMetadata,
        spl_2022_meta: TokenMetadata,
        accounts: &[solana_sdk::pubkey::Pubkey],
    ) -> arbitrary::Result<solana_sdk::instruction::Instruction> {
        match self {
            Self::SPLToken(i) => i.build(u, spl_meta, spl_2022_meta, accounts),
            Self::SPLToken2022(i) => i.build(u, spl_meta, spl_2022_meta, accounts),
            Self::System(i) => i.build(u, spl_meta, spl_2022_meta, accounts),
        }
    }
}
