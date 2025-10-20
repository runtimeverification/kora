#![no_main]

mod utils;
use arbitrary::Unstructured;
use kora_lib::{
    signer::{KoraSigner, SolanaMemorySigner},
    state::{get_config, update_config},
    tests::config_mock::ConfigMockBuilder,
    transaction::{VersionedTransactionOps, VersionedTransactionResolved},
};
use libfuzzer_sys::fuzz_target;
use litesvm::LiteSVM;
use litesvm_token;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::RpcClientConfig};
use solana_message::Message;
use solana_sdk::{
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, VersionedTransaction},
};
use spl_token_2022::instruction::transfer_checked;
use std::sync::{Arc, LazyLock};
use utils::LiteSVMSender;

static SVM_INIT: LazyLock<LiteSVM> = LazyLock::new(|| LiteSVM::new());

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);

    let signer = Keypair::new();
    let signer_pubkey = signer.pubkey();
    let alice = Keypair::new();
    let bob = Keypair::new();

    let mut svm = SVM_INIT.clone();
    let _ = svm.airdrop(&signer_pubkey, 1_000_000_000);
    let _ = svm.airdrop(&alice.pubkey(), 1_000_000_000);
    let _ = svm.airdrop(&bob.pubkey(), 1_000_000_000);
    let mut mint =
        litesvm_token::CreateMint::new(&mut svm, &signer).authority(&signer_pubkey).decimals(6);
    let token_pubkey = mint.send().expect("Couldn't create mint");

    let alice_ata =
        litesvm_token::CreateAssociatedTokenAccount::new(&mut svm, &signer, &token_pubkey)
            .owner(&alice.pubkey())
            .send()
            .expect("Couldn't create alice ata");
    let bob_ata =
        litesvm_token::CreateAssociatedTokenAccount::new(&mut svm, &signer, &token_pubkey)
            .owner(&bob.pubkey())
            .send()
            .expect("Couldn't create bob ata");
    let signer_ata =
        litesvm_token::CreateAssociatedTokenAccount::new(&mut svm, &signer, &token_pubkey)
            .owner(&signer_pubkey)
            .send()
            .expect("Couldn't create bob ata");

    litesvm_token::MintTo::new(&mut svm, &signer, &token_pubkey, &alice_ata, 100_000_000)
        .send()
        .expect("Couldn't mint to alice");
    litesvm_token::MintTo::new(&mut svm, &signer, &token_pubkey, &bob_ata, 100_000_000)
        .send()
        .expect("Couldn't mint to bob");

    // Create kora configuration
    let allowed_tokens = vec![token_pubkey.to_string()];
    let fuzzconfig = ConfigMockBuilder::new()
        .with_cache_enabled(false)
        .with_allowed_tokens(allowed_tokens.clone())
        .with_allowed_spl_paid_tokens(kora_lib::config::SplTokenConfig::Allowlist(allowed_tokens))
        .with_payment_address(Some(signer_pubkey.to_string()))
        .build();

    update_config(fuzzconfig).expect("Couldn't update global config");
    let config = get_config().expect("could not retrieve global config");

    let token_pgm: Pubkey = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap(); // Token Program

    let payment_ix = transfer_checked(
        &token_pgm,
        &alice_ata,
        &token_pubkey,
        &signer_ata,
        &signer_pubkey,
        &[],
        8_000_000,
        6,
    )
    .expect("Couldn't create token transfer instruction");
    //let tx = FuzzTransaction::arbitrary(&mut u)
    //    .expect("Can't build transaction")
    //    .build(None, Some(vec![payment_ix]));
    //let mut tx = VersionedTransactionResolved::from_kora_built_transaction(&tx);

    let message = Message::new(&[payment_ix], None);
    let transaction = Transaction::new_unsigned(message);
    let vt = VersionedTransaction::from(transaction.clone());
    let mut tx = VersionedTransactionResolved::from_kora_built_transaction(&vt);

    let sender = LiteSVMSender(svm);
    let rpc_config = RpcClientConfig::default();
    let rpc = RpcClient::new_sender(sender, rpc_config);

    let signer = Arc::new(KoraSigner::Memory(SolanaMemorySigner::new(signer)));

    let res = pollster::block_on(tx.sign_transaction_if_paid(&signer, &rpc));

    if let Err(err) = res {
        println!("Fuzz error");
        println!("Config: {:?}", config);
        println!("Pubkeys:");
        println!("    Bob: {:?}", bob.pubkey());
        println!("    Bob_ATA: {:?}", bob_ata);
        println!("    Alice: {:?}", alice.pubkey());
        println!("    Alice_ATA: {:?}", alice_ata);
        println!("    Signer: {:?}", signer_pubkey);
        //println!("Transaction: {:?}", transaction);
        println!("Error: {:?}", err);
        assert!(false);
    }

    drop(rpc);
});
