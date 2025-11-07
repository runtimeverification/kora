use arbitrary::{Result, Unstructured};
use kora_lib::signer::{KoraSigner, SignerPool, SignerWithMetadata, SolanaMemorySigner};
use litesvm::{types::FailedTransactionMetadata, LiteSVM};
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    transaction::Transaction,
};
use spl_token::id as spl_id;
use spl_token_2022::id as spl_2022_id;

pub const NUM_ACCOUNTS: usize = 4;

#[derive(Clone, Copy)]
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
    pub spl_2022_metadata: TokenMetadata,
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

        let spl_program = spl_id();
        let spl_2022_program = spl_2022_id();

        let [spl_metadata, spl_2022_metadata] =
            [spl_program, spl_2022_program].map(|token_program| {
                let mint = if token_program == spl_program {
                    litesvm_token::CreateMint::new(&mut svm, &kora_signer)
                        .authority(&kora_signer_pubkey)
                        .token_program_id(&token_program)
                        .decimals(decimals)
                        .send()
                        .unwrap()
                } else {
                    create_2022_mint(&mut svm, &kora_signer, decimals).unwrap()
                };

                let atas = pubkeys.map(|key| {
                    let ata = litesvm_token::CreateAssociatedTokenAccount::new(
                        &mut svm,
                        &kora_signer,
                        &mint,
                    )
                    .token_program_id(&token_program)
                    .owner(&key)
                    .send()
                    .unwrap();
                    let _ = if token_program == spl_program {
                        litesvm_token::MintTo::new(
                            &mut svm,
                            &kora_signer,
                            &mint,
                            &ata,
                            100_000_000,
                        )
                        .token_program_id(&token_program)
                        .send()
                        .unwrap();
                    } else {
                        mint_2022(&mut svm, &kora_signer, &mint, &ata, 100_000_000).unwrap();
                    };
                    ata
                });

                TokenMetadata {
                    program: token_program,
                    mint_pubkey: mint,
                    decimals: decimals,
                    atas: atas,
                    is_2022: token_program == spl_2022_program,
                }
            });

        Self {
            svm: svm,
            spl_metadata: spl_metadata,
            spl_2022_metadata: spl_2022_metadata,
            accounts: pubkeys,
            kora_signer: kora_signer,
        }
    }
}

fn create_2022_mint(
    svm: &mut LiteSVM,
    payer: &Keypair,
    decimals: u8,
) -> Result<Pubkey, FailedTransactionMetadata> {
    let mint_size = spl_token_2022::extension::ExtensionType::try_calculate_account_len::<
        spl_token_2022::state::Mint,
    >(&[])?;
    let mint_kp = Keypair::new();
    let mint_pk = mint_kp.pubkey();
    let token_program_id = spl_2022_id();
    let payer_pk = payer.pubkey();

    let ix1 = solana_sdk::system_instruction::create_account(
        &payer_pk,
        &mint_pk,
        svm.minimum_balance_for_rent_exemption(mint_size),
        mint_size as u64,
        &token_program_id,
    );
    let ix2 = spl_token_2022::instruction::initialize_mint2(
        &token_program_id,
        &mint_pk,
        &payer_pk,
        None,
        decimals,
    )?;

    let block_hash = svm.latest_blockhash();
    let tx = Transaction::new_signed_with_payer(
        &[ix1, ix2],
        Some(&payer_pk),
        &[payer, &mint_kp],
        block_hash,
    );
    svm.send_transaction(tx)?;

    Ok(mint_pk)
}

fn mint_2022(
    svm: &mut LiteSVM,
    payer: &Keypair,
    mint: &Pubkey,
    destination: &Pubkey,
    amount: u64,
) -> Result<(), FailedTransactionMetadata> {
    let payer_pk = payer.pubkey();
    let token_program_id = spl_2022_id();

    let authority = payer_pk;
    let signing_keys = [&payer_pk];
    let signer_keys = signing_keys.as_slice();

    let ix = spl_token_2022::instruction::mint_to(
        &token_program_id,
        mint,
        destination,
        &authority,
        &signer_keys,
        amount,
    )?;

    let block_hash = svm.latest_blockhash();
    let mut tx = Transaction::new_with_payer(&[ix], Some(&payer_pk));
    tx.partial_sign(&[payer], block_hash);

    svm.send_transaction(tx)?;

    Ok(())
}

pub trait BuildableInstruction {
    fn build(
        &self,
        u: &mut Unstructured,
        spl_meta: TokenMetadata,
        spl_2022_meta: TokenMetadata,
        accounts: &[Pubkey],
    ) -> Result<Instruction>;
}

pub fn build_signer_pool(kora_signer: Keypair) -> SignerPool {
    let signer = KoraSigner::Memory(SolanaMemorySigner::new(kora_signer));
    let signer_metadata = SignerWithMetadata::new("KoraSigner".parse().unwrap(), signer, 1);
    SignerPool::new(vec![signer_metadata])
}

pub trait FuzzUtils {
    fn token_balance(&self, ata: &Pubkey) -> std::result::Result<u64, String>;
}
