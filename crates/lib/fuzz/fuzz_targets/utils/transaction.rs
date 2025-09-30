use super::{spl_token as spl_token_fuzz, spl_token_2022 as spl_token_2022_fuzz};
use arbitrary::Arbitrary;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    message::Message,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, VersionedTransaction},
};
use spl_token::instruction::TokenInstruction;
use spl_token_2022::instruction::TokenInstruction as TokenInstruction2022;

#[derive(Arbitrary, Clone)]
pub enum FuzzInstruction<'a> {
    Legacy(spl_token_fuzz::FuzzTokenIx<'a>),
    Token2022(spl_token_2022_fuzz::FuzzToken2022Ix<'a>),
}

impl<'a> From<FuzzInstruction<'a>> for Vec<u8> {
    fn from(value: FuzzInstruction<'a>) -> Self {
        match value {
            FuzzInstruction::Legacy(token_ix) => {
                let token: TokenInstruction = token_ix.into();
                token.pack()
            }
            FuzzInstruction::Token2022(token_ix) => {
                let token: TokenInstruction2022 = token_ix.into();
                token.pack()
            }
        }
    }
}

#[derive(Arbitrary)]
pub struct FuzzTransaction<'a> {
    instructions: Vec<FuzzInstruction<'a>>,
}

impl<'a> FuzzTransaction<'a> {
    pub fn build(&self) -> VersionedTransaction {
        let keypair = Keypair::new();
        let account_metas = vec![AccountMeta::new(keypair.pubkey(), true)];
        let instructions: Vec<Instruction> = self
            .instructions
            .iter()
            .map(|a| {
                let data: Vec<u8> = (*a).clone().into();
                let program = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap();
                Instruction::new_with_bincode(program, &data, account_metas.clone())
            })
            .collect();
        let message = Message::new(instructions.as_slice(), Some(&keypair.pubkey()));
        let transaction = Transaction::new_unsigned(message);
        VersionedTransaction::from(transaction)
    }
}
