#![no_main]

mod utils;
use arbitrary::{Arbitrary, Unstructured};
use kora_lib::{
    signer::{KoraSigner, SolanaMemorySigner},
    state::update_config,
    tests::{rpc_mock::RpcMockBuilder, transaction_mock::create_mock_transaction},
    transaction::{VersionedTransactionOps, VersionedTransactionResolved},
};
use libfuzzer_sys::fuzz_target;
use solana_sdk::signature::Keypair;
use std::sync::Arc;
use utils::{FuzzConfig, FuzzTransaction};

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let fuzzconfig =
        FuzzConfig::arbitrary(&mut u).expect("Couldn't generate arbitrary fuzzing configuration");
    let config = fuzzconfig.kora_config;
    update_config(config).expect("Couldn't update global config");

    let transaction =
        FuzzTransaction::arbitrary(&mut u).expect("Couldn't generate arbitrary transaction");

    let mut tx = VersionedTransactionResolved::from_kora_built_transaction(&transaction.build());
    let rpc = RpcMockBuilder::new().build();

    let keypair = Keypair::new();
    let signer = Arc::new(KoraSigner::Memory(SolanaMemorySigner::new(keypair)));

    let _res = pollster::block_on(tx.sign_transaction_if_paid(&signer, &rpc));
});
