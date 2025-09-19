#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use kora_lib::{
    config,
    fee::price::{PriceConfig, PriceModel},
    signer::{KoraSigner, SolanaMemorySigner},
    state::update_config,
    tests::{
        config_mock::ConfigMockBuilder, rpc_mock::RpcMockBuilder,
        transaction_mock::create_mock_transaction,
    },
    transaction::{VersionedTransactionOps, VersionedTransactionResolved},
    Config,
};
use libfuzzer_sys::fuzz_target;
use solana_sdk::signature::Keypair;
use std::sync::Arc;

struct FuzzConfig {
    kora_config: Config,
}

impl<'a> Arbitrary<'a> for FuzzConfig {
    fn arbitrary(u: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let mut config = ConfigMockBuilder::new().build();

        // Validation config
        let validation = &mut config.validation;
        validation.max_allowed_lamports = u64::arbitrary(u)?;
        validation.max_signatures = u64::arbitrary(u)?;

        validation.fee_payer_policy = config::FeePayerPolicy::arbitrary(u)?;

        let n: u64 = u.arbitrary()?;
        let margin = (n as f64) / (u64::MAX as f64);
        validation.price = PriceConfig { model: PriceModel::Margin { margin: margin } };

        validation.token_2022 = config::Token2022Config::arbitrary(u)?;

        Ok(Self { kora_config: config })
    }
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let fuzzconfig =
        FuzzConfig::arbitrary(&mut u).expect("Couldn't generate arbitrary fuzzing configuration");
    let config = fuzzconfig.kora_config;
    update_config(config).expect("Couldn't update global config");

    let transaction = create_mock_transaction();

    let mut tx = VersionedTransactionResolved::from_kora_built_transaction(&transaction);
    let rpc = RpcMockBuilder::new().build();

    let keypair = Keypair::new();
    let signer = Arc::new(KoraSigner::Memory(SolanaMemorySigner::new(keypair)));

    let _res = pollster::block_on(tx.sign_transaction_if_paid(&signer, &rpc));
});
