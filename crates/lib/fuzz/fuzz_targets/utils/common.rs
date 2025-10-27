use arbitrary::{Result, Unstructured};
use litesvm::LiteSVM;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer};
use spl_token::id as spl_id;
use spl_token_2022::id as spl_2022_id;

pub const NUM_ACCOUNTS: usize = 4;

#[derive(Clone)]
pub struct TokenMetadata {
    pub program: Pubkey,
    pub mint_pubkey: Pubkey,
    pub decimals: u8,
    pub atas: [Pubkey; NUM_ACCOUNTS],
    pub is_2022: bool,
}

pub struct InitialState {
    pub svm: LiteSVM,
    pub spl_metadata: TokenMetadata,
    pub accounts: [Pubkey; NUM_ACCOUNTS],
    pub kora_signer: Keypair,
}

impl InitialState {
    pub fn new() -> Self {
        let decimals = 6_u8;

        let mut svm = LiteSVM::new().with_sigverify(false);
        let keys = [(); NUM_ACCOUNTS].map(|_| Keypair::new());
        let kora_signer = keys[0].insecure_clone();
        let pubkeys = keys.map(|keypair| keypair.pubkey());
        for pubkey in pubkeys {
            let _ = svm.airdrop(&pubkey, 1_000_000_000);
        }

        let kora_signer_pubkey = kora_signer.pubkey();

        let [spl_metadata] = [spl_id() /* , spl_2022_id()*/].map(|token_program| {
            let mint = litesvm_token::CreateMint::new(&mut svm, &kora_signer)
                .authority(&kora_signer_pubkey)
                .token_program_id(&token_program)
                .decimals(decimals)
                .send()
                .unwrap();

            let atas = pubkeys.map(|key| {
                let ata =
                    litesvm_token::CreateAssociatedTokenAccount::new(&mut svm, &kora_signer, &mint)
                        .owner(&key)
                        .send()
                        .unwrap();
                let _ =
                    litesvm_token::MintTo::new(&mut svm, &kora_signer, &mint, &ata, 100_000_000)
                        .send()
                        .unwrap();
                ata
            });

            TokenMetadata {
                program: token_program,
                mint_pubkey: mint,
                decimals: decimals,
                atas: atas,
                is_2022: token_program == spl_2022_id(),
            }
        });

        Self { svm: svm, spl_metadata: spl_metadata, accounts: pubkeys, kora_signer: kora_signer }
    }
}

pub trait BuildableInstruction {
    fn build(
        &self,
        u: &mut Unstructured,
        token: &Pubkey,
        accounts: &[Pubkey],
        atas: &[Pubkey],
    ) -> Result<Instruction>;
}
