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
        // step 0: create product publish without model create should fail
        /*assert_err!(
            create_product_publish(
                app_id,
                document_id,
                model_id,
                product_id,
                "hash",
                para_issue_rate,
                self_issue_rate
            ),
            Error::<Test>::ModelNotFound
        );

        // create model type
        assert_ok!(create_model_type(model_type_id, "abc".as_bytes().to_vec()));
        // set model size limit
        assert_ok!(set_model_size_limit(app_id, 100));

        assert!(expect_doc_power == doc_power.content);


        // step 5_1, this comment should trigger account power update
        let alice_account_power =
            KpModule::kp_auth_account_power(alice_signer_pair.public().into());
        //let bob_account_power = KpModule::kp_auth_account_power(bob_signer_pair.public().into());
        //let tom_account_power = KpModule::kp_auth_account_power(tom_signer_pair.public().into());
        assert!(alice_account_power > 0);*/

        let balance: u64 = 1000;
        let factor = KpModule::power_factor(455200000); // 45.52
        let converted = KpModule::balance_apply_power(balance, factor);
        assert!(converted == 1u64);
    });
}
