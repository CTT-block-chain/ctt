#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use codec::Codec;
use kp::LeaderBoardResult;
use primitives::PowerSize;
use sp_std::prelude::*;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait KpApi<AccountId, Balance> where AccountId: Codec, Balance: Codec {
        fn total_power() -> PowerSize;
        fn account_power(account: AccountId) -> PowerSize;
        fn commodity_power(app_id: u32, cart_id: Vec<u8>) -> PowerSize;
        fn document_power(app_id: u32, doc_id: Vec<u8>) -> PowerSize;
        fn is_commodity_power_exist(app_id: u32, cart_id: Vec<u8>) -> bool;
        fn leader_board_result(block: u32, app_id: u32, model_id: Vec<u8>) -> LeaderBoardResult<AccountId>;
        fn stake_to_vote(account: AccountId, stake: Balance) -> Balance;
    }
}
