use async_trait::async_trait;
use litesvm::LiteSVM;
use serde_json::json;
use solana_client::{
    client_error::Result,
    rpc_request::RpcRequest,
    rpc_sender::{RpcSender, RpcTransportStats},
};

struct LiteSVMSender<'a>(&'a LiteSVM);

#[async_trait]
impl<'a> RpcSender for LiteSVMSender<'a> {
    async fn send(
        &self,
        request: RpcRequest,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let method = format!("{request}");
        let val = match method.as_str() {
            "getAccountInfo" => todo!("get_account"),
            "getBalance" => todo!("get_balance"),
            "getLatestBlockhash" => todo!("latest_blockhash"),
            "getTransaction" => todo!("get_transaction"),
            "sendTransaction" => todo!("send_transaction"),
            "simulateTransaction" => todo!("simulate_transaction"),
            _ => todo!("unsupported method"),
        };
        Ok(val)
    }

    fn get_transport_stats(&self) -> RpcTransportStats {
        RpcTransportStats::default()
    }

    fn url(&self) -> String {
        String::default()
    }
}
