#![no_main]

mod utils;
use arbitrary::Unstructured;
use kora_lib::{
    config::FeePayerPolicy,
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

use crate::utils::{spl_token::FuzzSPLInstruction, spl_token_2022::FuzzSPL2022Instruction};

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
        .with_allowed_programs(vec![
            "11111111111111111111111111111111".parse().unwrap(), // System Program
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap(), // Token Program
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".parse().unwrap(), // Token-2022 Program
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse().unwrap(), // ATA Program
        ])
        .with_fee_payer_policy(FeePayerPolicy {
            allow_token2022_transfers: false,
            ..Default::default()
        })
        .build();

    update_config(fuzzconfig).expect("Couldn't update global config");

    let token_pgm: Pubkey = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap(); // Token Program

    let payment_ix = transfer_checked(
        &token_pgm,
        &alice_ata,
        &token_pubkey,
        &signer_ata,
        &signer_pubkey,
        &[],
        5000,
        6,
    )
    .expect("Couldn't create token transfer instruction");

    //let token2022transfer = FuzzSPL2022Instruction::Transfer.build(&mut u, token_pubkey, accounts.as_slice(), atas.as_slice()).unwrap();
    //let token2022transfer = spl_token_2022::instruction::transfer_checked(&spl_token_2022::id(), bob_ata, &signer_pubkey, token_pubkey, &[], 5000, decimals).unwrap();
    let token2022transfer = spl_token_2022::instruction::transfer_checked(
        &spl_token_2022::id(),
        bob_ata,
        token_pubkey,
        signer_ata,
        &signer_pubkey,
        &[],
        5000,
        *decimals,
    )
    .unwrap();
    let n = u.int_in_range(0..=20).unwrap();
    let extra_instrs: Vec<FuzzInstruction> =
        u.arbitrary_iter::<FuzzInstruction>().unwrap().take(n).map(|i| i.unwrap()).collect();
    let extra_instrs: Vec<Instruction> = extra_instrs
        .iter()
        .map(|instr| {
            instr.build(&mut u, token_pubkey, accounts.as_slice(), atas.as_slice()).unwrap()
        })
        .collect();
    let mut all_instrs: Vec<Instruction> =
        [&[token2022transfer, payment_ix], extra_instrs.as_slice()].concat();
    let all_instrs: &mut [Instruction] = all_instrs.as_mut_slice();
    for i in (1..all_instrs.len()).rev() {
        let j = u.int_in_range(0..=i).unwrap();
        all_instrs.swap(i, j);
    }

    let message = Message::new(all_instrs, None);
    let transaction = Transaction::new_unsigned(message);
    let vt = VersionedTransaction::from(transaction.clone());
    let mut tx = VersionedTransactionResolved::from_kora_built_transaction(&vt);

    let sender = LiteSVMSender(svm);
    let rpc_config = RpcClientConfig::default();
    let rpc = RpcClient::new_sender(sender, rpc_config);

    let signer =
        Arc::new(KoraSigner::Memory(SolanaMemorySigner::new(kora_signer.insecure_clone())));

    let res = pollster::block_on(tx.sign_transaction_if_paid(&signer, &rpc));

    if let Ok(res) = res {
        println!("Fuzz error");
        println!("System instructions: {:?}", tx.get_or_parse_system_instructions());
        println!("SPL instructions: {:?}", tx.get_or_parse_spl_instructions());
        println!("Transaction: {:?}", tx.clone());
        assert!(false);
    }
});
