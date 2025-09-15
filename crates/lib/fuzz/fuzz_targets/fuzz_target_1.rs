#![no_main]

use kora_lib::signer::SolanaMemorySigner;
use libfuzzer_sys::fuzz_target;
use solana_sdk::signature::Keypair;
use kora_lib::{Config, KoraError};
use kora_lib::signer::KoraSigner;
use kora_lib::state::update_config;
use kora_lib::tests::config_mock::ConfigMockBuilder;
use kora_lib::tests::transaction_mock::create_mock_transaction;
use kora_lib::tests::rpc_mock::RpcMockBuilder;
use kora_lib::transaction::{VersionedTransactionOps, VersionedTransactionResolved};
use std::sync::Arc;

fuzz_target!(|data: &[u8]| {
    let config: Config = ConfigMockBuilder::new().build();
    if update_config(config).is_err() {
        panic!("Can't update config");
    }

    let transaction = create_mock_transaction();

    let mut tx = VersionedTransactionResolved::from_kora_built_transaction(&transaction);
    let rpc = RpcMockBuilder::new().build();

    let keypair = Keypair::new();
    let signer = Arc::new(KoraSigner::Memory(SolanaMemorySigner::new(keypair)));

    let res = pollster::block_on(tx.sign_transaction_if_paid(&signer, &rpc));

    if res.is_err() {
        panic!("Error signing! {res:?}");
    }
});
