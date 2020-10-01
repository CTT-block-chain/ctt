use frame_support::{assert_err, assert_ok, dispatch};
use sp_core::H256;
use sp_io::hashing::blake2_256;

use crate::mock::*;
use crate::*;
use sp_core::{sr25519, Pair};

use primitives::PowerSize;

#[test]
fn kp_account_power() {
    new_test_ext().execute_with(|| {
        // define some shared vars
        let alice_signer_pair =
            sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");
        let bob_signer_pair =
            sr25519::Pair::from_string(&format!("//{}", "Bob"), None).expect("valid seed");
        let tom_signer_pair =
            sr25519::Pair::from_string(&format!("//{}", "Tom"), None).expect("valid seed");
        let app_id: u32 = 100;
        let model_id = "m01";
        let document_id = "d01";
        let product_id = "p01";
        let para_issue_rate: PowerSize = 1088;
        let self_issue_rate: PowerSize = 3024;

        // step 0: create product publish without model create should fail
        assert_err!(
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

        // step 1: create product model
        assert_ok!(create_model(
            app_id, model_id, "e01", "name", "type", "hash"
        ));

        // step 1_1: create repeat model should fail
        assert_err!(
            create_model(app_id, model_id, "e01", "name", "type", "hash"),
            Error::<Test>::ModelAlreadyExisted
        );

        // step 2: create product publish parameters
        assert_ok!(create_product_publish(
            app_id,
            document_id,
            model_id,
            product_id,
            "hash",
            para_issue_rate,
            self_issue_rate
        ));

        // step 2_1 create repeat product_publish should fail
        assert_err!(
            create_product_publish(
                app_id,
                document_id,
                model_id,
                product_id,
                "hash",
                para_issue_rate,
                self_issue_rate
            ),
            Error::<Test>::DocumentAlreadyExisted
        );

        // step 3 verify product publish records correct knowledge power
        // only one doc, so max is self
        let mut expect_doc_power = ((1.0
            * <Test as Trait>::DocumentPublishWeightParamsRate::get() as f64
            / RATIO_DIV
            + 1.0 * <Test as Trait>::DocumentPublishWeightParamsSelfRate::get() as f64 / RATIO_DIV)
            * <Test as Trait>::DocumentPowerWeightContent::get() as f64
            / RATIO_DIV
            * <Test as Trait>::TopWeightProductPublish::get() as f64
            / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize;

        let mut doc_power = KpModule::kp_document_power(
            app_id,
            document_id.as_bytes().to_vec(),
        );
        assert!(expect_doc_power == doc_power.content);
        let product_publish_power = doc_power;

        // step 3_1 create another publish document which cause rate change
        let new_para_issue_rate = 673;
        let new_self_issue_rate = 856;
        assert_ok!(create_product_publish(
            app_id,
            "d02",
            model_id,
            "p02",
            "hash",
            new_para_issue_rate,
            new_self_issue_rate
        ));

        expect_doc_power = ((new_para_issue_rate as f64
            / new_para_issue_rate.max(para_issue_rate) as f64
            * <Test as Trait>::DocumentPublishWeightParamsRate::get() as f64
            / RATIO_DIV
            + new_self_issue_rate as f64 / new_self_issue_rate.max(self_issue_rate) as f64
                * <Test as Trait>::DocumentPublishWeightParamsSelfRate::get() as f64
                / RATIO_DIV)
            * <Test as Trait>::DocumentPowerWeightContent::get() as f64
            / RATIO_DIV
            * <Test as Trait>::TopWeightProductPublish::get() as f64
            / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize;

        doc_power =
            KpModule::kp_document_power(app_id, "d02".as_bytes().to_vec());
        assert!(expect_doc_power == doc_power.content);

        // step 4 create product identify document
        let goods_price: PowerSize = 1000; // 10yuan
        let ident_rate: PowerSize = 2538;
        let ident_consistence: PowerSize = 3427;
        let cart_id = "c01";
        let product_identify_document_id = "pi01";
        assert_ok!(create_product_identify(
            app_id,
            product_identify_document_id,
            model_id,
            product_id,
            "hash",
            goods_price,
            ident_rate,
            ident_consistence,
            cart_id
        ));

        // step 4_1 check if generate correct account power
        // here should only has content power
        expect_doc_power = ((1.0 * <Test as Trait>::DocumentIdentifyWeightParamsRate::get() as f64
            / RATIO_DIV
            + 1.0 * <Test as Trait>::DocumentIdentifyWeightCheckRate::get() as f64 / RATIO_DIV)
            * <Test as Trait>::DocumentPowerWeightContent::get() as f64
            / RATIO_DIV
            * <Test as Trait>::TopWeightDocumentIdentify::get() as f64
            / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize;

        doc_power = KpModule::kp_document_power(
            app_id,
            product_identify_document_id.as_bytes().to_vec(),
        );
        assert!(expect_doc_power == doc_power.content);
        // this doc power also be doc creator account power now
        let account_power = KpModule::kp_auth_account_power(alice_signer_pair.public().into());
        assert!(product_publish_power.total() + doc_power.total() == account_power);

        // step 5 create a normal comment on this document
        let comment_fee: PowerSize = 100; // 1yuan
        let comment_id = "c01";
        assert_ok!(create_comment(
            "Alice",
            app_id,
            product_identify_document_id,
            comment_id,
            comment_fee,
            "hash",
            0
        ));

        assert_ok!(create_comment(
            "Bob",
            app_id,
            product_identify_document_id,
            "c02",
            200,
            "hash",
            1
        ));

        assert_ok!(create_comment(
            "Tom",
            app_id,
            product_identify_document_id,
            "c03",
            150,
            "hash",
            0
        ));

        // step 5_1, this comment should trigger account power update
        let alice_account_power =
            KpModule::kp_auth_account_power(alice_signer_pair.public().into());
        //let bob_account_power = KpModule::kp_auth_account_power(bob_signer_pair.public().into());
        //let tom_account_power = KpModule::kp_auth_account_power(tom_signer_pair.public().into());
        assert!(alice_account_power > 0);
    });
}

fn create_product_publish(
    app_id: u32,
    document_id: &str,
    model_id: &str,
    product_id: &str,
    hash_content: &str,
    para_issue_rate: PowerSize,
    self_issue_rate: PowerSize,
) -> dispatch::DispatchResult {
    let test_hash = H256::from_slice(&blake2_256(hash_content.as_bytes()));
    let test_signer_pair =
        sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

    let app_id_vec = app_id;
    let model_id_vec = model_id.as_bytes().to_vec();
    let product_id_vec = product_id.as_bytes().to_vec();
    let document_id_vec = document_id.as_bytes().to_vec();

    // TODO: sign
    let test_signature = test_signer_pair.sign("abc".as_bytes());

    KpModule::create_product_publish_document(
        Origin::signed(1),
        app_id_vec,
        document_id_vec,
        model_id_vec,
        product_id_vec,
        test_hash,
        KPProductPublishData {
            para_issue_rate,
            self_issue_rate,
        },
        test_signer_pair.public().clone().into(),
        test_signature.clone(),
        test_signer_pair.public().into(),
        test_signature,
    )
}

fn create_product_identify(
    app_id: u32,
    document_id: &str,
    model_id: &str,
    product_id: &str,
    hash_content: &str,
    goods_price: PowerSize,
    ident_rate: PowerSize,
    ident_consistence: PowerSize,
    cart_id: &str,
) -> dispatch::DispatchResult {
    let test_hash = H256::from_slice(&blake2_256(hash_content.as_bytes()));
    let test_signer_pair =
        sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

    let app_id_vec = app_id;
    let model_id_vec = model_id.as_bytes().to_vec();
    let product_id_vec = product_id.as_bytes().to_vec();
    let document_id_vec = document_id.as_bytes().to_vec();
    let cart_id_vec = cart_id.as_bytes().to_vec();

    // TODO: sign
    let test_signature = test_signer_pair.sign("abc".as_bytes());

    KpModule::create_product_identify_document(
        Origin::signed(1),
        app_id_vec,
        document_id_vec,
        model_id_vec,
        product_id_vec,
        test_hash,
        KPProductIdentifyData {
            goods_price,
            ident_rate,
            ident_consistence,
            cart_id: cart_id_vec,
        },
        test_signer_pair.public().clone().into(),
        test_signature.clone(),
        test_signer_pair.public().into(),
        test_signature,
    )
}

fn create_product_try(
    app_id: u32,
    document_id: &str,
    model_id: &str,
    product_id: &str,
    hash_content: &str,
    goods_price: PowerSize,
    offset_rate: PowerSize,
    true_rate: PowerSize,
    cart_id: &str,
) -> dispatch::DispatchResult {
    let test_hash = H256::from_slice(&blake2_256(hash_content.as_bytes()));
    let test_signer_pair =
        sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

    let app_id_vec = app_id;
    let model_id_vec = model_id.as_bytes().to_vec();
    let product_id_vec = product_id.as_bytes().to_vec();
    let document_id_vec = document_id.as_bytes().to_vec();
    let cart_id_vec = cart_id.as_bytes().to_vec();

    // TODO: sign
    let test_signature = test_signer_pair.sign("abc".as_bytes());

    KpModule::create_product_try_document(
        Origin::signed(1),
        app_id_vec,
        document_id_vec,
        model_id_vec,
        product_id_vec,
        test_hash,
        KPProductTryData {
            goods_price,
            offset_rate,
            true_rate,
            cart_id: cart_id_vec,
        },
        test_signer_pair.public().clone().into(),
        test_signature.clone(),
        test_signer_pair.public().into(),
        test_signature,
    )
}

fn create_comment(
    owner_seed: &str,
    app_id: u32,
    document_id: &str,
    comment_id: &str,
    comment_fee: PowerSize,
    hash_content: &str,
    comment_trend: u8,
) -> dispatch::DispatchResult {
    let test_hash = H256::from_slice(&blake2_256(hash_content.as_bytes()));
    let test_signer_pair =
        sr25519::Pair::from_string(&format!("//{}", owner_seed), None).expect("valid seed");

    let app_id_vec = app_id;
    let comment_id_vec = comment_id.as_bytes().to_vec();
    let document_id_vec = document_id.as_bytes().to_vec();

    // TODO: sign
    let test_signature = test_signer_pair.sign("abc".as_bytes());

    KpModule::create_comment(
        Origin::signed(1),
        app_id_vec,
        comment_id_vec,
        document_id_vec,
        test_hash,
        comment_fee,
        comment_trend,
        test_signer_pair.public().clone().into(),
        test_signature.clone(),
        test_signer_pair.public().into(),
        test_signature,
    )
}

fn create_model(
    app_id: u32,
    model_id: &str,
    expert_id: &str,
    commodity_name: &str,
    commodity_type: &str,
    hash_content: &str,
) -> dispatch::DispatchResult {
    let test_hash = H256::from_slice(&blake2_256(hash_content.as_bytes()));
    let test_signer_pair =
        sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

    let app_id_vec = app_id;
    let model_id_vec = model_id.as_bytes().to_vec();
    let expert_id_vec = expert_id.as_bytes().to_vec();
    let commodity_name_vec = commodity_name.as_bytes().to_vec();
    let commodity_type_vec = commodity_type.as_bytes().to_vec();

    let mut buf = vec![];
    //buf.append(&mut (app_id_vec.clone()));
    buf.append(&mut (model_id_vec.clone()));

    let test_signature = test_signer_pair.sign(&buf);

    KpModule::create_model(
        Origin::signed(1),
        app_id_vec,
        model_id_vec,
        expert_id_vec,
        commodity_name_vec,
        10001,
        test_hash,
        test_signer_pair.public().clone().into(),
        test_signature.clone(),
        test_signer_pair.public().into(),
        test_signature,
    )
}
