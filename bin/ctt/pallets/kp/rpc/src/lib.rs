//! RPC interface for the kp module.

use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use kp_runtime_api::KpApi as KpRuntimeApi;
use primitives::AuthAccountId;
use serde::{Deserialize, Serialize};
use sp_api::ProvideRuntimeApi;
use sp_blockchain::HeaderBackend;
use sp_core::Bytes;
use sp_runtime::{generic::BlockId, traits::Block as BlockT};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct QueryCommodityPowerParams {
    app_id: Bytes,
    cart_id: Bytes,
}

#[rpc]
pub trait KpApi<BlockHash, AccountId> {
    #[rpc(name = "kP_totalPower")]
    fn total_power(&self, at: Option<BlockHash>) -> Result<u32>;

    #[rpc(name = "kP_accountPower")]
    fn account_power(&self, account: AccountId, at: Option<BlockHash>) -> Result<u32>;

    #[rpc(name = "kP_commodityPower")]
    fn commodity_power(
        &self,
        query: QueryCommodityPowerParams,
        at: Option<BlockHash>,
    ) -> Result<u32>;
}

/// A struct that implements the `KpApi`.
pub struct Kp<C, M> {
    // If you have more generics, no need to SumStorage<C, M, N, P, ...>
    // just use a tuple like SumStorage<C, (M, N, P, ...)>
    client: Arc<C>,
    _marker: std::marker::PhantomData<M>,
}

impl<C, M> Kp<C, M> {
    /// Create new `Kp` instance with the given reference to the client.
    pub fn new(client: Arc<C>) -> Self {
        Self {
            client,
            _marker: Default::default(),
        }
    }
}

/// Error type of this RPC api.
// pub enum Error {
// 	/// The transaction was not decodable.
// 	DecodeError,
// 	/// The call to runtime failed.
// 	RuntimeError,
// }
//
// impl From<Error> for i64 {
// 	fn from(e: Error) -> i64 {
// 		match e {
// 			Error::RuntimeError => 1,
// 			Error::DecodeError => 2,
// 		}
// 	}
// }

impl<C, Block> KpApi<<Block as BlockT>::Hash, AuthAccountId> for Kp<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: KpRuntimeApi<Block, AuthAccountId>,
{
    fn total_power(&self, at: Option<<Block as BlockT>::Hash>) -> Result<u32> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.total_power(&at);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn account_power(
        &self,
        account: AuthAccountId,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<u32> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let runtime_api_result = api.account_power(&at, account);
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn commodity_power(
        &self,
        query: QueryCommodityPowerParams,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<u32> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let QueryCommodityPowerParams { app_id, cart_id } = query;

        let runtime_api_result = api.commodity_power(&at, app_id.to_vec(), cart_id.to_vec());
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
