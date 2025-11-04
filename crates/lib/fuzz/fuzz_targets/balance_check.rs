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
    common::{build_signer_pool, BuildableInstruction, InitialState},
    FuzzInstruction, LiteSVMSender,
};

static SVM_INIT: LazyLock<InitialState> = LazyLock::new(|| InitialState::new());

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let InitialState { svm, spl_metadata, spl_2022_metadata, accounts, kora_signer } = &*SVM_INIT;
    let mut svm = Arc::new((*svm).clone());

    let allowed_tokens =
        vec![spl_metadata.mint_pubkey.to_string(), spl_2022_metadata.mint_pubkey.to_string()];
    let config = ConfigMockBuilder::new()
        .with_cache_enabled(false)
        .with_allowed_tokens(allowed_tokens.clone())
        .with_allowed_spl_paid_tokens(SplTokenConfig::Allowlist(allowed_tokens))
        .build();
    let pool = build_signer_pool(kora_signer.insecure_clone());

    update_config(config).unwrap();
    update_signer_pool(pool).unwrap();
    let _ = pollster::block_on(UsageTracker::init_usage_limiter());

    let n = u.int_in_range(1..=20).unwrap();
    let ixs: Vec<FuzzInstruction> =
        u.arbitrary_iter::<FuzzInstruction>().unwrap().take(n).map(|i| i.unwrap()).collect();
    let mut ixs: Vec<Instruction> = ixs
        .iter()
        .map(|i| i.build(&mut u, *spl_metadata, *spl_2022_metadata, accounts.as_slice()).unwrap())
        .collect();

    let payment_ix = spl_token::instruction::transfer_checked(
        &spl_metadata.program,
        &spl_metadata.atas[1],
        &spl_metadata.mint_pubkey,
        &spl_metadata.atas[0],
        &accounts[1],
        &[],
        1_000_000,
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

    let sender = LiteSVMSender(svm.clone());
    let rpc_config = RpcClientConfig::default();
    let rpc_client = RpcClient::new_sender(sender, rpc_config);
    let rpc = KoraRpc::new(Arc::new(rpc_client));

    let res = pollster::block_on(rpc.sign_transaction_if_paid(request));

    if let Ok(res) = res {

    }
});
