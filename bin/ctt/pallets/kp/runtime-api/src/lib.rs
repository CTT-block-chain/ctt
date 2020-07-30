#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::too_many_arguments)]

use codec::Codec;
use sp_core::Bytes;
use sp_std::prelude::*;

// Here we declare the runtime API. It is implemented it the `impl` block in
// runtime amalgamator file (the `runtime/src/lib.rs`)
sp_api::decl_runtime_apis! {
    pub trait KpApi<AccountId> where AccountId: Codec {
        fn total_power() -> u32;
        fn account_power(account: AccountId) -> u32;
        fn commodity_power(app_id: Vec<u8>, cart_id: Vec<u8>) -> u32;
    }
}
