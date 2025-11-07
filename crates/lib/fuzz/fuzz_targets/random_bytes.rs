#![no_main]

mod utils;
use std::{
    fmt,
    sync::{Arc, LazyLock},
};

use arbitrary::{Arbitrary, Unstructured};
use base64::Engine;
use kora_lib::{
    rpc_server::{method::sign_transaction_if_paid::SignTransactionIfPaidRequest, KoraRpc},
    state::{update_config, update_signer_pool},
    tests::config_mock::ConfigMockBuilder,
    transaction::TransactionUtil,
    usage_limit::UsageTracker,
};
use libfuzzer_sys::fuzz_target;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::RpcClientConfig};
use utils::common::InitialState;

use crate::utils::{common::build_signer_pool, LiteSVMSender};

static INIT_SVM: LazyLock<InitialState> = LazyLock::new(|| InitialState::new());

struct FuzzSignTransactionIfPaidRequest(SignTransactionIfPaidRequest);

impl<'a> Arbitrary<'a> for FuzzSignTransactionIfPaidRequest {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        u.arbitrary::<Vec<u8>>().and_then(|bytes| {
            let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
            Ok(Self(SignTransactionIfPaidRequest {
                transaction: encoded,
                signer_key: None,
                sig_verify: true,
            }))
        })
    }
}

impl fmt::Debug for FuzzSignTransactionIfPaidRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let request @ SignTransactionIfPaidRequest { transaction, signer_key: _, sig_verify: _ } =
            &self.0;
        let versioned = TransactionUtil::decode_b64_transaction(transaction.as_str())
            .map_err(|_| fmt::Error)?;
        write!(f, "{versioned:#?}\n{request:#?}")
    }
}

fuzz_target!(|data: FuzzSignTransactionIfPaidRequest| {
    let InitialState { svm, kora_signer, .. } = &*INIT_SVM;
    let svm = Arc::new((*svm).clone());

    let config = ConfigMockBuilder::new().with_cache_enabled(false).build();
    let pool = build_signer_pool(kora_signer.insecure_clone());

    update_config(config).unwrap();
    update_signer_pool(pool).unwrap();
    let _ = pollster::block_on(UsageTracker::init_usage_limiter());

    let sender = LiteSVMSender(svm);
    let rpc_config = RpcClientConfig::default();
    let rpc_client = RpcClient::new_sender(sender, rpc_config);

    let rpc = KoraRpc::new(Arc::new(rpc_client));

    let request = data.0;
    let _res = pollster::block_on(rpc.sign_transaction_if_paid(request));
});
