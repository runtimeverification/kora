use async_trait::async_trait;
use litesvm::LiteSVM;
use serde_json::json;
use solana_account_decoder::{encode_ui_account, UiAccountEncoding};
use solana_client::{
    client_error::Result,
    rpc_request::RpcRequest,
    rpc_response::{Response, RpcResponseContext},
    rpc_sender::{RpcSender, RpcTransportStats},
};
use solana_sdk::pubkey::Pubkey;

pub struct LiteSVMSender(pub LiteSVM);

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
            "simulateTransaction" => todo!("simulate_transaction"),
            "getFeeForMessage" => json!(5000 as u64),
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
