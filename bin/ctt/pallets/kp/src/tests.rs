use frame_support::{assert_err, assert_ok, dispatch};
use frame_system::RawOrigin;
use sp_core::H256;
use sp_io::hashing::blake2_256;

use crate::mock::*;
use crate::*;
use sp_core::{sr25519, Pair};

use primitives::{Balance, PowerSize};

#[test]
fn kp_account_power() {
    new_test_ext().execute_with(|| {
        let balance: u64 = 1000_000_000;
        let factor = KpModule::power_factor(200000);
        let converted = KpModule::balance_apply_power(balance, factor);
        assert!(converted == 3252747248u64);
    });
}
