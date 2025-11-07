#![no_main]

mod utils;
use arbitrary::Unstructured;
use kora_lib::{
    config::FeePayerPolicy,
    rpc_server::{method::sign_transaction_if_paid::SignTransactionIfPaidRequest, KoraRpc},
    state::{update_config, update_signer_pool},
    tests::config_mock::ConfigMockBuilder,
    transaction::{VersionedTransactionOps, VersionedTransactionResolved},
    usage_limit::UsageTracker,
};
use libfuzzer_sys::fuzz_target;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::RpcClientConfig};
use solana_message::Message;
use solana_sdk::{
    instruction::Instruction,
    signer::Signer,
    transaction::{Transaction, VersionedTransaction},
};
use spl_token_2022::instruction::transfer_checked;
use std::sync::{Arc, LazyLock};
use utils::{
    common::{BuildableInstruction, InitialState},
    FuzzInstruction, LiteSVMSender,
};

use crate::utils::common::{build_signer_pool, TokenMetadata};

static SVM_INIT: LazyLock<InitialState> = LazyLock::new(|| InitialState::new());

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);

    let InitialState { svm, spl_metadata, spl_2022_metadata, accounts, kora_signer } = &*SVM_INIT;
    let svm = Arc::new((*svm).clone()); // Very important to clone here for an iteration-specific instance of the vm

    // Create kora configuration
    let allowed_tokens =
        vec![spl_metadata.mint_pubkey.to_string(), spl_2022_metadata.mint_pubkey.to_string()];
    let fuzzconfig = ConfigMockBuilder::new()
        .with_cache_enabled(false)
        .with_allowed_tokens(allowed_tokens.clone())
        .with_allowed_spl_paid_tokens(kora_lib::config::SplTokenConfig::Allowlist(allowed_tokens))
        .with_payment_address(Some(kora_signer.pubkey().to_string()))
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

    let pool = build_signer_pool(kora_signer.insecure_clone());

    update_config(fuzzconfig).unwrap();
    update_signer_pool(pool).unwrap();
    let _ = pollster::block_on(UsageTracker::init_usage_limiter());

    let TokenMetadata { program, mint_pubkey, decimals, atas, .. } = spl_metadata;

    // Create a payment instruction
    let payment_ix = transfer_checked(
        &program,
        &atas[2],
        &mint_pubkey,
        &atas[0],
        &accounts[2],
        &[],
        5000,
        *decimals,
    )
    .unwrap();

    let TokenMetadata { program, mint_pubkey, decimals, atas, .. } = spl_2022_metadata;
    let payment_2022_ix = spl_token_2022::instruction::transfer_checked(
        &program,
        &atas[2],
        &mint_pubkey,
        &atas[0],
        &accounts[2],
        &[],
        5000,
        *decimals,
    )
    .unwrap();

    // Create extra arbitrary instructions
    let n = u.int_in_range(0..=20).unwrap();
    let extra_instrs: Vec<FuzzInstruction> =
        u.arbitrary_iter::<FuzzInstruction>().unwrap().take(n).map(|i| i.unwrap()).collect();
    let extra_instrs: Vec<Instruction> = extra_instrs
        .iter()
        .map(|instr| {
            instr.build(&mut u, *spl_metadata, *spl_2022_metadata, accounts.as_slice()).unwrap()
        })
        .collect();

    // Shuffle all of the instructions around
    let mut all_instrs: Vec<Instruction> =
        [&[payment_ix, payment_2022_ix], extra_instrs.as_slice()].concat();
    let all_instrs: &mut [Instruction] = all_instrs.as_mut_slice();
    for i in (1..all_instrs.len()).rev() {
        let j = u.int_in_range(0..=i).unwrap();
        all_instrs.swap(i, j);
    }

    // Build a transaction with the instructions
    let message = Message::new_with_blockhash(
        all_instrs,
        Some(&kora_signer.pubkey()),
        &svm.latest_blockhash(),
    );
    let transaction = Transaction::new_unsigned(message);
    let vt = VersionedTransaction::from(transaction.clone());
    let tx = VersionedTransactionResolved::from_kora_built_transaction(&vt);

    let sender = LiteSVMSender(svm);
    let rpc_config = RpcClientConfig::default();
    let rpc_client = RpcClient::new_sender(sender, rpc_config);

    let rpc = KoraRpc::new(Arc::new(rpc_client));

    let request = SignTransactionIfPaidRequest {
        transaction: tx.encode_b64_transaction().unwrap(),
        signer_key: None,
        sig_verify: false,
    };
    let res = pollster::block_on(rpc.sign_transaction_if_paid(request));

    if let Ok(res) = res {
        panic!();
    }
});
