use arbitrary::{Arbitrary, Result, Unstructured};
use kora_lib::state::get_config;
use litesvm::LiteSVM;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer};
use std::{str::FromStr, sync::LazyLock};

static FUZZ_KEYS: LazyLock<Vec<Pubkey>> = LazyLock::new(|| {
    let count = 5;
    let mut res = Vec::with_capacity(count);
    for _ in 1..=count {
        res.push(Pubkey::new_unique());
    }

    let config = get_config().expect("Could not retrieve global config");
    if let Some(payment_address) = &config.kora.payment_address {
        let signer_key =
            Pubkey::from_str(payment_address.as_str()).expect("Could not parse payment address");
        res.push(signer_key);
    }

    res
});

pub struct InitialState {
    pub svm: LiteSVM,
    pub accounts: [Keypair; 3],
    pub atas: [Pubkey; 3],
    pub kora_signer: Keypair,
    pub kora_ata: Pubkey,
    pub token: Pubkey,
    pub decimals: u8,
}

impl InitialState {
    pub fn new() -> Self {
        let kora_signer = Keypair::new();
        let kora_signer_pubkey = kora_signer.pubkey();
        let alice = Keypair::new();
        let bob = Keypair::new();
        let mavory = Keypair::new();
        let decimals = 6_u8;

        let mut svm = LiteSVM::new();
        let _ = svm.airdrop(&kora_signer_pubkey, 1_000_000_000);
        let _ = svm.airdrop(&alice.pubkey(), 1_000_000_000);
        let _ = svm.airdrop(&bob.pubkey(), 1_000_000_000);
        let _ = svm.airdrop(&mavory.pubkey(), 1_000_000_000);
        let mint = litesvm_token::CreateMint::new(&mut svm, &kora_signer)
            .authority(&kora_signer_pubkey)
            .decimals(decimals);
        let token_pubkey = mint.send().expect("Couldn't create mint");

        let alice_ata =
            litesvm_token::CreateAssociatedTokenAccount::new(&mut svm, &kora_signer, &token_pubkey)
                .owner(&alice.pubkey())
                .send()
                .expect("Couldn't create alice ata");
        let bob_ata =
            litesvm_token::CreateAssociatedTokenAccount::new(&mut svm, &kora_signer, &token_pubkey)
                .owner(&bob.pubkey())
                .send()
                .expect("Couldn't create bob ata");
        let mavory_ata =
            litesvm_token::CreateAssociatedTokenAccount::new(&mut svm, &kora_signer, &token_pubkey)
                .owner(&mavory.pubkey())
                .send()
                .expect("Couldn't create mavory ata");
        let kora_signer_ata =
            litesvm_token::CreateAssociatedTokenAccount::new(&mut svm, &kora_signer, &token_pubkey)
                .owner(&kora_signer_pubkey)
                .send()
                .expect("Couldn't create bob ata");

        litesvm_token::MintTo::new(&mut svm, &kora_signer, &token_pubkey, &alice_ata, 100_000_000)
            .send()
            .expect("Couldn't mint to alice");
        litesvm_token::MintTo::new(&mut svm, &kora_signer, &token_pubkey, &bob_ata, 100_000_000)
            .send()
            .expect("Couldn't mint to bob");
        litesvm_token::MintTo::new(&mut svm, &kora_signer, &token_pubkey, &mavory_ata, 100_000_000)
            .send()
            .expect("Couldn't mint to mavory");
        litesvm_token::MintTo::new(
            &mut svm,
            &kora_signer,
            &token_pubkey,
            &kora_signer_ata,
            100_000_000,
        )
        .send()
        .expect("Couldn't mint to mavory");

        Self {
            svm: svm,
            accounts: [alice, bob, mavory],
            atas: [alice_ata, bob_ata, mavory_ata],
            kora_signer: kora_signer,
            kora_ata: kora_signer_ata,
            token: token_pubkey,
            decimals: decimals,
        }
    }
}

#[derive(Clone, Debug)]
pub struct FuzzPubkey(Pubkey);

impl<'a> Arbitrary<'a> for FuzzPubkey {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let keys = &FUZZ_KEYS;
        let index = u.int_in_range(1..=keys.len())? - 1;
        Ok(Self(keys[index]))
    }
}

impl From<FuzzPubkey> for Pubkey {
    fn from(value: FuzzPubkey) -> Self {
        value.0
    }
}

pub trait BuildableInstruction {
    fn build(
        &self,
        u: &mut Unstructured,
        token: &Pubkey,
        accounts: &[&Pubkey],
        atas: &[&Pubkey],
    ) -> Result<Instruction>;
}
