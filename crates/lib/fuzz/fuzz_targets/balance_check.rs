#![no_main]

mod utils;
use std::sync::{Arc, LazyLock};

use arbitrary::Unstructured;
use kora_lib::{
    config::SplTokenConfig,
    rpc_server::{method::sign_transaction_if_paid::SignTransactionIfPaidRequest, KoraRpc},
    state::{update_config, update_signer_pool},
    tests::config_mock::ConfigMockBuilder,
    transaction::{VersionedTransactionOps, VersionedTransactionResolved},
    usage_limit::UsageTracker,
};
use libfuzzer_sys::fuzz_target;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::RpcClientConfig};
use solana_sdk::{
    instruction::Instruction,
    signer::Signer,
    transaction::{Transaction, VersionedTransaction},
};

use crate::utils::{
    common::{build_signer_pool, BuildableInstruction, FuzzUtils, InitialState},
    FuzzInstruction, LiteSVMSender,
};

static SVM_INIT: LazyLock<InitialState> = LazyLock::new(|| InitialState::new());

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let InitialState { svm, spl_metadata, spl_2022_metadata, accounts, kora_signer } = &*SVM_INIT;
    let mut svm = Arc::new((*svm).clone());

    // Set up the Kora config
    let allowed_tokens =
        vec![spl_metadata.mint_pubkey.to_string(), spl_2022_metadata.mint_pubkey.to_string()];
    let config = ConfigMockBuilder::new()
        .with_cache_enabled(false)
        .with_allowed_tokens(allowed_tokens.clone())
        .with_allowed_spl_paid_tokens(SplTokenConfig::Allowlist(allowed_tokens))
        .with_allowed_programs(vec![
            "11111111111111111111111111111111".parse().unwrap(), // System Program
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA".parse().unwrap(), // Token Program
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".parse().unwrap(), // Token-2022 Program
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL".parse().unwrap(), // ATA Program
        ])
        .build();
    let pool = build_signer_pool(kora_signer.insecure_clone());

    update_config(config).unwrap();
    update_signer_pool(pool).unwrap();
    let _ = pollster::block_on(UsageTracker::init_usage_limiter());

    let sender = LiteSVMSender(svm.clone());
    let rpc_config = RpcClientConfig::default();
    let rpc_client = RpcClient::new_sender(sender, rpc_config);

    // Create the random instructions
    let n = u.int_in_range(10..=50).unwrap();
    let ixs: Vec<FuzzInstruction> =
        u.arbitrary_iter::<FuzzInstruction>().unwrap().take(n).map(|i| i.unwrap()).collect();
    let mut ixs: Vec<Instruction> = ixs
        .iter()
        .map(|i| i.build(&mut u, *spl_metadata, *spl_2022_metadata, accounts.as_slice()).unwrap())
        .collect();

    // Get the message fee for the transaction w/ random instructions
    let mut fee_transaction =
        Transaction::new_with_payer(ixs.as_slice(), Some(&kora_signer.pubkey()));
    fee_transaction.message.recent_blockhash = svm.latest_blockhash();
    let fee_message = fee_transaction.message;
    let fee = pollster::block_on(rpc_client.get_fee_for_message(&fee_message)).unwrap();

    // Create the payment instruction based off of the fee
    let payment_ix = spl_token::instruction::transfer_checked(
        &spl_metadata.program,
        &spl_metadata.atas[1],
        &spl_metadata.mint_pubkey,
        &spl_metadata.atas[0], // Index 0 = Kora signer's account
        &accounts[1],
        &[],
        fee,
        spl_metadata.decimals,
    )
    .unwrap();
    ixs.insert(0, payment_ix);

    let mut transaction: Transaction =
        Transaction::new_with_payer(ixs.as_slice(), Some(&kora_signer.pubkey()));
    transaction.message.recent_blockhash = svm.latest_blockhash();
    let versioned_transaction = VersionedTransaction::from(transaction);
    let transaction =
        VersionedTransactionResolved::from_kora_built_transaction(&versioned_transaction);

    let request = SignTransactionIfPaidRequest {
        transaction: transaction.encode_b64_transaction().unwrap(),
        signer_key: None,
        sig_verify: false,
    };

    let rpc = KoraRpc::new(Arc::new(rpc_client));

    let res = pollster::block_on(rpc.sign_transaction_if_paid(request));

    if let Ok(res) = res {
        let mut svm = (*svm).clone();

        let kora_ata = spl_metadata.atas[0]; // Index 0 = Kora signer's account
        let kora_lamports_before = svm.get_account(&accounts[0]).unwrap().lamports;
        let spl_tokens_before = svm.token_balance(&kora_ata).unwrap() * 1_000_000;

        let _tx_res = svm.send_transaction(transaction.transaction.clone());

        let kora_lamports = svm.get_account(&accounts[0]).unwrap().lamports;
        let spl_tokens = svm.token_balance(&kora_ata).unwrap() * 1_000_000;

        // Calculate balance as an aggregate of lamport balances + token balances.
        let initial_balance: i64 = (kora_lamports_before + spl_tokens_before) as i64;
        let after_balance: i64 = (kora_lamports + spl_tokens) as i64;

        if after_balance < initial_balance {
            println!("{:#?}", transaction.transaction);
            println!(
                "Balances:\n    total before/after/diff: {} {} {}",
                initial_balance,
                after_balance,
                after_balance - initial_balance
            );
            println!(
                "Balances:\n    lamports before/after/diff: {} {} {}",
                kora_lamports_before,
                kora_lamports,
                (kora_lamports as i64) - (kora_lamports_before as i64)
            );
            println!(
                "Balances:\n    spl_tokens before/after/diff: {} {} {}",
                spl_tokens_before,
                spl_tokens,
                (spl_tokens as i64) - (spl_tokens_before as i64)
            );
            assert!(false);
        }
    }
});
