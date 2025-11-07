use std::sync::Arc;

use async_trait::async_trait;
use base64::{engine::general_purpose::STANDARD, Engine};
use kora_lib::transaction::TransactionUtil;
use litesvm::{
    types::{FailedTransactionMetadata, SimulatedTransactionInfo, TransactionMetadata},
    LiteSVM,
};
use serde_json::json;
use solana_account_decoder::{encode_ui_account, UiAccount, UiAccountData, UiAccountEncoding};
use solana_client::{
    client_error::Result,
    rpc_config::RpcSimulateTransactionConfig,
    rpc_request::RpcRequest,
    rpc_response::{Response, RpcResponseContext, RpcSimulateTransactionResult},
    rpc_sender::{RpcSender, RpcTransportStats},
};
use solana_sdk::{
    account::{Account, AccountSharedData},
    clock::Clock,
    pubkey::Pubkey,
};
use solana_transaction_status_client_types::{
    InnerInstruction, InnerInstructions, UiInnerInstructions,
};

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
                        let ui_account = encode_ui_account(
                            &pubkey,
                            &acct,
                            UiAccountEncoding::Base64,
                            None,
                            None,
                        );
                        serde_json::to_value(ui_account)?
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
                    let simulated_result = self.0.simulate_transaction(versioned);

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
                        err: res.err(),
                        logs: Some(logs),
                        accounts: accounts,
                        units_consumed: Some(compute_units_consumed),
                        loaded_accounts_data_size: None, // ?
                        return_data: Some(return_data.into()),
                        inner_instructions: Some(ui_instructions),
                        replacement_blockhash: None,
                    })
                    .unwrap()
                } else {
                    serde_json::Value::Null
                }
            }
            "getFeeForMessage" => json!(5000 as u64),
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
        .map(|(_, account_shared_data)| {
            let account: Account = account_shared_data.clone().into();
            let data =
                UiAccountData::Binary(STANDARD.encode(account.data), UiAccountEncoding::Base64);
            Some(UiAccount {
                lamports: account.lamports,
                data: data,
                owner: account.owner.to_string(),
                executable: account.executable,
                rent_epoch: account.rent_epoch,
                space: None,
            })
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
