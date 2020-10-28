//! RPC interface for the kp module.

pub use self::gen_client::Client as KpClient;
use jsonrpc_core::{Error as RpcError, ErrorCode, Result};
use jsonrpc_derive::rpc;
use kp::LeaderBoardResult;
use kp_runtime_api::KpApi as KpRuntimeApi;
pub use kp_runtime_api::KpApi as KpRuntimeRpcApi;
use primitives::{AuthAccountId, PowerSize};
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
    app_id: u32,
    cart_id: Bytes,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct QueryLeaderBoardParams {
    app_id: u32,
    model_id: Bytes,
    block: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct LeaderBoardItemRPC<AccountId> {
    cart_id: Bytes,
    power: PowerSize,
    owner: AccountId,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct LeaderBoardResultRPC<AccountId> {
    accounts: Vec<AccountId>,
    board: Vec<LeaderBoardItemRPC<AccountId>>,
}

#[rpc]
pub trait KpApi<BlockHash, AccountId> {
    #[rpc(name = "kP_totalPower")]
    fn total_power(&self, at: Option<BlockHash>) -> Result<PowerSize>;

    #[rpc(name = "kP_accountPower")]
    fn account_power(&self, account: AccountId, at: Option<BlockHash>) -> Result<PowerSize>;

    #[rpc(name = "kP_commodityPower")]
    fn commodity_power(
        &self,
        query: QueryCommodityPowerParams,
        at: Option<BlockHash>,
    ) -> Result<PowerSize>;

    #[rpc(name = "kP_isCommodityPowerExist")]
    fn is_commodity_power_exist(
        &self,
        query: QueryCommodityPowerParams,
        at: Option<BlockHash>,
    ) -> Result<bool>;

    #[rpc(name = "kP_leaderBoardResult")]
    fn leader_board_result(
        &self,
        query: QueryLeaderBoardParams,
        at: Option<BlockHash>,
    ) -> Result<LeaderBoardResultRPC<AccountId>>;
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
pub enum Error {
    /// The transaction was not decodable.
    DecodeError,
    /// The call to runtime failed.
    RuntimeError,
}

impl From<Error> for i64 {
    fn from(e: Error) -> i64 {
        match e {
            Error::RuntimeError => 1,
            Error::DecodeError => 2,
        }
    }
}

impl<C, Block> KpApi<<Block as BlockT>::Hash, AuthAccountId> for Kp<C, Block>
where
    Block: BlockT,
    C: Send + Sync + 'static,
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C::Api: KpRuntimeRpcApi<Block, AuthAccountId>,
{
    fn total_power(&self, at: Option<<Block as BlockT>::Hash>) -> Result<PowerSize> {
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
    ) -> Result<PowerSize> {
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
    ) -> Result<PowerSize> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let QueryCommodityPowerParams { app_id, cart_id } = query;

        let runtime_api_result = api.commodity_power(&at, app_id, cart_id.to_vec());
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn is_commodity_power_exist(
        &self,
        query: QueryCommodityPowerParams,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<bool> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let QueryCommodityPowerParams { app_id, cart_id } = query;

        let runtime_api_result = api.is_commodity_power_exist(&at, app_id, cart_id.to_vec());
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }

    fn leader_board_result(
        &self,
        query: QueryLeaderBoardParams,
        at: Option<<Block as BlockT>::Hash>,
    ) -> Result<LeaderBoardResultRPC<AuthAccountId>> {
        let api = self.client.runtime_api();
        let at = BlockId::hash(at.unwrap_or_else(||
            // If the block hash is not supplied assume the best block.
            self.client.info().best_hash));

        let QueryLeaderBoardParams {
            app_id,
            model_id,
            block,
        } = query;

        let runtime_api_result = api.leader_board_result(&at, block, app_id, model_id.to_vec());
        runtime_api_result.map_err(|e| RpcError {
            code: ErrorCode::ServerError(9876), // No real reason for this value
            message: "Something wrong".into(),
            data: Some(format!("{:?}", e).into()),
        })
    }
}
