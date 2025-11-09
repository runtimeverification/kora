use std::{collections::HashSet, sync::Arc};

use super::common::FuzzUtils;
use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use kora_lib::transaction::TransactionUtil;
use litesvm::{
    types::{FailedTransactionMetadata, SimulatedTransactionInfo, TransactionMetadata},
    LiteSVM,
};
use serde_json::json;
//use solana_account_decoder::{encode_ui_account, UiAccount, UiAccountData, UiAccountEncoding};
use solana_account_decoder::{encode_ui_account, UiAccount, UiAccountEncoding, UiDataSliceConfig};
use solana_client::{
    client_error::Result,
    rpc_config::RpcSimulateTransactionConfig,
    rpc_request::RpcRequest,
    rpc_response::{Response, RpcResponseContext, RpcSimulateTransactionResult},
    rpc_sender::{RpcSender, RpcTransportStats},
};
use solana_commitment_config::CommitmentConfig;
use solana_fee::{calculate_fee, FeeFeatures};
use solana_message::{
    SanitizedMessage, SanitizedVersionedMessage, SimpleAddressLoader, VersionedMessage,
};
use solana_sdk::{
    account::{Account, AccountSharedData},
    clock::Clock,
    program_pack::Pack,
    pubkey::Pubkey,
};
use solana_transaction_status_client_types::{
    InnerInstruction, InnerInstructions, UiInnerInstructions,
};
use spl_token_2022_interface::{state::Account as SplAccount2022, ID as ID_2022};
use spl_token_interface::{state::Account as SplAccount, ID};

impl FuzzUtils for LiteSVM {
    fn token_balance(&self, ata: &Pubkey) -> std::result::Result<u64, String> {
        let account_data = self.get_account(ata).ok_or("Couldn't retrieve account".to_string())?;
        if account_data.owner == ID {
            let spl_account =
                SplAccount::unpack(&account_data.data).map_err(|err| format!("{err}"))?;
            Ok(spl_account.amount)
        } else if account_data.owner == ID_2022 {
            let spl_account =
                SplAccount2022::unpack(&account_data.data).map_err(|err| format!("{err}"))?;
            Ok(spl_account.amount)
        } else {
            let err_string = format!(
                "Account is not owned by a token program\nOwner: {}, ID: {}, ID_2022: {}",
                account_data.owner, ID, ID_2022
            );
            Err(err_string)
        }
    }
}

pub struct LiteSVMSender(pub Arc<LiteSVM>);

#[async_trait]
impl RpcSender for LiteSVMSender {
    async fn send(
        &self,
        request: RpcRequest,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let method = format!("{request}");
        let param_str = format!("{params}");
        let val = match method.as_str() {
            "getAccountInfo" => {
                let acct_str = params.as_array().unwrap()[0].as_str().unwrap();
                let pubkey = Pubkey::from_str_const(acct_str);
                let acct = self.0.get_account(&pubkey);
                match acct {
                    Some(acct) => {
                        //let ui_account = encode_ui_account(
                        //    &pubkey,
                        //    &acct,
                        //    UiAccountEncoding::Base64,
                        //    None,
                        //    None,
                        //);
                        //serde_json::to_value(ui_account)?
                        serde_json::to_value(acct)?
                    }
                    None => serde_json::Value::Null,
                }
            }
            "getBalance" => todo!("get_balance"),
            "getLatestBlockhash" => todo!("latest_blockhash"),
            "getTransaction" => todo!("get_transaction"),
            "sendTransaction" => todo!("send_transaction"),
            "simulateTransaction" => {
                let config = serde_json::from_value::<(String, Option<RpcSimulateTransactionConfig>)>(
                    params,
                );
                if let Ok((tx, _config)) = config {
                    let versioned = TransactionUtil::decode_b64_transaction(&tx).unwrap();
                    let simulated_result = self.0.simulate_transaction(versioned.clone());

                    let (metadata, res) = match simulated_result {
                        Ok(SimulatedTransactionInfo { meta, post_accounts }) => {
                            (meta, Ok(post_accounts))
                        }
                        Err(FailedTransactionMetadata { err, meta }) => (meta, Err(err)),
                    };

                    let TransactionMetadata {
                        signature: _,
                        logs,
                        inner_instructions,
                        compute_units_consumed,
                        return_data,
                    } = metadata;

                    let ui_instructions =
                        inner_instructions_to_ui_instructions(&inner_instructions);
                    let accounts = res.as_ref().ok().map(post_accounts_to_ui_accounts);

                    serde_json::to_value(RpcSimulateTransactionResult {
                        err: res.err().map(|e| e.into()),
                        logs: Some(logs),
                        accounts: accounts,
                        units_consumed: Some(compute_units_consumed),
                        loaded_accounts_data_size: None, // ?
                        return_data: Some(return_data.into()),
                        inner_instructions: Some(ui_instructions),
                        replacement_blockhash: None,
                        fee: None,
                        pre_balances: None,
                        post_balances: None,
                        pre_token_balances: None,
                        post_token_balances: None,
                        loaded_addresses: None,
                    })
                    .unwrap()
                } else {
                    serde_json::Value::Null
                }
            }
            "getFeeForMessage" => {
                let config = serde_json::from_value::<(String, Option<CommitmentConfig>)>(params);
                if let Ok((message_string, commitment_config)) = config {
                    let message_bytes = STANDARD.decode(message_string).unwrap();
                    let message =
                        bincode::deserialize::<VersionedMessage>(message_bytes.as_slice()).unwrap();
                    let sanitized_versioned_message =
                        SanitizedVersionedMessage::try_new(message).unwrap();
                    let sanitized_message = SanitizedMessage::try_new(
                        sanitized_versioned_message,
                        SimpleAddressLoader::Disabled,
                        &HashSet::new(),
                    )
                    .unwrap();
                    let fee_features = FeeFeatures { enable_secp256r1_precompile: true };
                    let fee = calculate_fee(&sanitized_message, false, 5000, 0, fee_features);
                    json!(fee)
                } else {
                    serde_json::Value::Null
                }
            }
            "getEpochInfo" => {
                let clock: Clock = self.0.get_sysvar::<Clock>();
                let slot = clock.slot;
                let epoch = clock.epoch;
                json!({
                    "result": {
                        "absoluteSlot": slot,
                        "blockHeight": 166500, // Fake number
                        "epoch": epoch,
                        "slotIndex": 2790, // Fake number
                        "slotsInEpoch": 8192, // Fake number
                        "transactionCount": 22661093 // Fake number
                    },
                })
            }
            _ => todo!("unsupported method, {}, {}", method, param_str),
        };
        Ok(json!(Response {
            context: RpcResponseContext { slot: 1, api_version: None },
            value: val
        }))
    }

    fn get_transport_stats(&self) -> RpcTransportStats {
        RpcTransportStats::default()
    }

    fn url(&self) -> String {
        String::default()
    }
}

fn post_accounts_to_ui_accounts(
    metadata: &Vec<(Pubkey, AccountSharedData)>,
) -> Vec<Option<UiAccount>> {
    metadata
        .iter()
        .map(|(address, account_shared_data)| {
            let account = encode_ui_account(
                address,
                account_shared_data,
                UiAccountEncoding::Binary,
                None,
                None,
            );
            Some(account)
        })
        .collect()
}

fn inner_instructions_to_ui_instructions(
    inner_instructions: &Vec<Vec<solana_message::inner_instruction::InnerInstruction>>,
) -> Vec<UiInnerInstructions> {
    inner_instructions
        .iter()
        .map(|ixs| {
            ixs.iter()
                .map(
                    // Convert LiteSVM's solana_message InnerInstruction to
                    // the solana_rpc_types InnerInstruction
                    |solana_message::inner_instruction::InnerInstruction {
                         instruction,
                         stack_height,
                     }| InnerInstruction {
                        instruction: instruction.clone(),
                        stack_height: Some(*stack_height as u32),
                    },
                )
                .collect::<Vec<InnerInstruction>>()
        })
        .enumerate()
        .map(|(i, x)| InnerInstructions { index: i as u8, instructions: x.clone() }.into())
        .collect()
}
