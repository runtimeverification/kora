#![no_main]

mod utils;
use std::{
    env,
    fmt::{self, Pointer},
    sync::{Arc, LazyLock},
};

use arbitrary::{Arbitrary, Unstructured};
use base64::Engine;
use kora_lib::{
    rpc_server::{method::sign_transaction_if_paid::SignTransactionIfPaidRequest, KoraRpc},
    signer::{signer::Signer, KoraSigner, SignerPool, SignerWithMetadata, SolanaMemorySigner},
    state::{init_signer_pool, update_config, update_signer_pool},
    tests::config_mock::{get_default_signer_pool_config, ConfigMockBuilder},
    transaction::{TransactionUtil, VersionedTransactionOps, VersionedTransactionResolved},
    usage_limit::UsageTracker,
};
use libfuzzer_sys::fuzz_target;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client::RpcClientConfig};
use solana_message::{compiled_instruction::CompiledInstruction, Message, MessageHeader};
use solana_sdk::{
    hash::{Hash, HASH_BYTES},
    pubkey::Pubkey,
    signature::Keypair,
    transaction::{Transaction, VersionedTransaction},
};
use utils::common::InitialState;

use crate::utils::LiteSVMSender;

static INIT_SVM: LazyLock<InitialState> = LazyLock::new(|| InitialState::new());

#[derive(Debug)]
struct FuzzMessage(pub Message);

impl<'a> Arbitrary<'a> for FuzzMessage {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let message = Message {
            header: MessageHeader {
                num_required_signatures: u.int_in_range(0..=u8::MAX).unwrap(),
                num_readonly_signed_accounts: u.int_in_range(0..=u8::MAX).unwrap(),
                num_readonly_unsigned_accounts: u.int_in_range(0..=u8::MAX).unwrap(),
            },
            account_keys: u
                .int_in_range(0..=10)
                .and_then(|len| {
                    u.arbitrary_iter::<[u8; 32]>()
                        .unwrap()
                        .take(len)
                        .map(|bytes| Ok(Pubkey::new_from_array(bytes.unwrap())))
                        .collect()
                })
                .unwrap(),
            recent_blockhash: Hash::new_from_array(u.arbitrary::<[u8; HASH_BYTES]>().unwrap()),
            instructions: u
                .int_in_range(0..=10)
                .and_then(|len| {
                    u.arbitrary_iter::<(u8, Vec<u8>, Vec<u8>)>()
                        .unwrap()
                        .take(len)
                        .map(|data| {
                            let (program_id_index, data, accounts) = data.unwrap();
                            Ok(CompiledInstruction::new_from_raw_parts(
                                program_id_index,
                                data,
                                accounts,
                            ))
                        })
                        .collect()
                })
                .unwrap(),
        };
        Ok(Self(message))
    }
}

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
    let svm = (*svm).clone();

    let config = ConfigMockBuilder::new().with_cache_enabled(false).build();
    let signer = KoraSigner::Memory(SolanaMemorySigner::new(kora_signer.insecure_clone()));
    let signer_metadata = SignerWithMetadata::new("KoraSigner".parse().unwrap(), signer, 1);
    let pool = SignerPool::new(vec![signer_metadata]);

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
