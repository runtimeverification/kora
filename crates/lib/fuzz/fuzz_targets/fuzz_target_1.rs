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
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::{Transaction, VersionedTransaction},
};
use spl_token_2022::instruction::transfer_checked;
use std::sync::{Arc, LazyLock};
use utils::{
    common::{BuildableInstruction, InitialState},
    FuzzInstruction, LiteSVMSender,
};

use crate::utils::spl_token::FuzzSPLInstruction;

static SVM_INIT: LazyLock<InitialState> = LazyLock::new(|| InitialState::new());

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);

    let InitialState {
        svm,
        accounts: [alice, bob, mavory],
        atas: [alice_ata, bob_ata, mavory_ata],
        kora_signer,
        kora_ata: signer_ata,
        token: token_pubkey,
        decimals,
    } = &*SVM_INIT;
    let svm = (*svm).clone(); // Very important to clone here for an iteration-specific instance of the vm
    let signer_pubkey = kora_signer.pubkey();
    let accounts = [&alice.pubkey(), &bob.pubkey(), &mavory.pubkey(), &signer_pubkey];
    let atas = [alice_ata, bob_ata, mavory_ata, signer_ata];

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
        //u.int_in_range(0..=8_000_000).unwrap(),
        5000,
        6,
    )
    .expect("Couldn't create token transfer instruction");

    let extra_instrs =
        u.arbitrary::<Vec<FuzzInstruction>>().expect("Couldn't create extra instructions");
    let mut more_instrs: Vec<Instruction> = extra_instrs
        .iter()
        .map(|instr| {
            instr.build(&mut u, &token_pubkey, accounts.as_slice(), atas.as_slice()).expect("asdf")
        })
        .collect();
    more_instrs.extend_from_slice(&[payment_ix]);

    let message = Message::new(more_instrs.as_slice(), None);
    let transaction = Transaction::new_unsigned(message);
    let vt = VersionedTransaction::from(transaction.clone());
    let mut tx = VersionedTransactionResolved::from_kora_built_transaction(&vt);

    let sender = LiteSVMSender(svm);
    let rpc_config = RpcClientConfig::default();
    let rpc = RpcClient::new_sender(sender, rpc_config);

    let signer =
        Arc::new(KoraSigner::Memory(SolanaMemorySigner::new(kora_signer.insecure_clone())));

    let res = pollster::block_on(tx.sign_transaction_if_paid(&signer, &rpc));

    if let Err(err) = res {
        let errstring = err.to_string();
        if !errstring.starts_with("Invalid transaction: Insufficient token payment") {
            println!("Fuzz error");
            println!("Config: {:?}", config);
            println!("Pubkeys:");
            println!("    Bob: {:?}", bob.pubkey());
            println!("    Bob_ATA: {:?}", bob_ata);
            println!("    Alice: {:?}", alice.pubkey());
            println!("    Alice_ATA: {:?}", alice_ata);
            println!("    Signer: {:?}", signer_pubkey);
            //println!("Transaction: {:?}", transaction);
            println!("{errstring}");
            assert!(false);
        }
    }
});
