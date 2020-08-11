use frame_support::{assert_err, assert_ok, dispatch};
use sp_core::H256;
use sp_io::hashing::blake2_256;

use crate::mock::*;
use crate::*;
use sp_core::{sr25519, Pair};

use sp_runtime::traits::{BlakeTwo256, Hash};

#[test]
fn kp_test_product_publish() {
    new_test_ext().execute_with(|| {
        let test_hash = H256::from_slice(&blake2_256(String::from("da038934asd1").as_bytes()));
        let test_signer_pair =
            sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

        let app_id = String::from("A01").into_bytes();
        let knowledge_id = String::from("K01").into_bytes();
        let knowledge_type: u8 = 0;

        let mut buf = vec![];
        buf.append(&mut (app_id.clone()));
        buf.append(&mut (knowledge_id.clone()));
        buf.append(&mut vec![knowledge_type]);

        let test_signature = test_signer_pair.sign(&buf);

        let para_issue_rate: u32 = 32;
        let self_issue_rate: u32 = 45;

        assert_ok!(KpModule::create_product_publish_document(
            Origin::signed(1),
            app_id.clone(),
            knowledge_id.clone(),
            String::from("M01").into_bytes(),
            String::from("P01").into_bytes(),
            test_hash,
            KPProductPublishData {
                para_issue_rate,
                self_issue_rate
            },
            test_signer_pair.public().clone().into(),
            test_signature.clone(),
            test_signer_pair.public().into(),
            test_signature,
        ));
        // asserting that the stored value is equal to what we stored
        let doc_key_hash = BlakeTwo256::hash_of(&(&app_id, &knowledge_id));
        let read = KpModule::kp_document_data_by_idhash(&doc_key_hash);
        println!("read result:{} {}", read.owner, read.content_hash);

        assert_eq!(
            read.document_data,
            DocumentSpecificData::ProductPublish(KPProductPublishData {
                para_issue_rate,
                self_issue_rate,
            })
        );

        // this is first item max

        let power = (10000 * 60 + 10000 * 40) * 30 * 15 / 1000000;

        let read_power = KpModule::kp_document_power_by_idhash(&doc_key_hash);

        println!("expected power:{}", power);
        assert_eq!(read_power.content, power);
    });
}

#[test]
fn kp_test_product_identify() {
    new_test_ext().execute_with(|| {
        let test_hash = H256::from_slice(&blake2_256(String::from("da038934asd1").as_bytes()));
        let test_signer_pair =
            sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

        let app_id = String::from("A01").into_bytes();
        let knowledge_id = String::from("K01").into_bytes();
        let knowledge_type: u8 = 1;

        let mut buf = vec![];
        buf.append(&mut (app_id.clone()));
        buf.append(&mut (knowledge_id.clone()));
        buf.append(&mut vec![knowledge_type]);

        let test_signature = test_signer_pair.sign(&buf);

        let goods_price: u32 = 100;
        let ident_rate: u32 = 10;
        let ident_consistence: u32 = 23;
        let cart_id: Vec<u8> = String::from("C01").into_bytes();

        assert_ok!(KpModule::create_product_identify_document(
            Origin::signed(1),
            app_id.clone(),
            knowledge_id.clone(),
            String::from("M01").into_bytes(),
            String::from("P01").into_bytes(),
            test_hash,
            KPProductIdentifyData {
                goods_price,
                ident_rate,
                ident_consistence,
                cart_id: cart_id.clone(),
            },
            test_signer_pair.public().clone().into(),
            test_signature.clone(),
            test_signer_pair.public().into(),
            test_signature,
        ));
        // asserting that the stored value is equal to what we stored
        let doc_key_hash = BlakeTwo256::hash_of(&(&app_id, &knowledge_id));
        let read = KpModule::kp_document_data_by_idhash(&doc_key_hash);
        println!("read result:{} {}", read.owner, read.content_hash);

        assert_eq!(
            read.document_data,
            DocumentSpecificData::ProductIdentify(KPProductIdentifyData {
                goods_price,
                ident_rate,
                ident_consistence,
                cart_id,
            })
        );

        // this is first item max

        /*let power = (10000 * 60 + 10000 * 40) * 30 * 15 / 1000000;

        let read_power = KpModule::kp_document_power_by_idhash(&doc_key_hash);

        println!("expected power:{}", power);
        assert_eq!(read_power.content, power);*/
    });
}

#[test]
fn kp_test_product_try() {
    new_test_ext().execute_with(|| {
        let test_hash = H256::from_slice(&blake2_256(String::from("da038934asd1").as_bytes()));
        let test_signer_pair =
            sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

        let app_id = String::from("A01").into_bytes();
        let knowledge_id = String::from("K01").into_bytes();
        let knowledge_type: u8 = 2;

        let mut buf = vec![];
        buf.append(&mut (app_id.clone()));
        buf.append(&mut (knowledge_id.clone()));
        buf.append(&mut vec![knowledge_type]);

        let test_signature = test_signer_pair.sign(&buf);

        let goods_price: u32 = 100;
        let offset_rate: u32 = 33;
        let true_rate: u32 = 21;
        let cart_id: Vec<u8> = String::from("C01").into_bytes();

        assert_ok!(KpModule::create_product_try_document(
            Origin::signed(1),
            app_id.clone(),
            knowledge_id.clone(),
            String::from("M01").into_bytes(),
            String::from("P01").into_bytes(),
            test_hash,
            KPProductTryData {
                goods_price,
                offset_rate,
                true_rate,
                cart_id: cart_id.clone(),
            },
            test_signer_pair.public().clone().into(),
            test_signature.clone(),
            test_signer_pair.public().into(),
            test_signature,
        ));
        // asserting that the stored value is equal to what we stored
        let doc_key_hash = BlakeTwo256::hash_of(&(&app_id, &knowledge_id));
        let read = KpModule::kp_document_data_by_idhash(&doc_key_hash);
        println!("read result:{} {}", read.owner, read.content_hash);

        assert_eq!(
            read.document_data,
            DocumentSpecificData::ProductTry(KPProductTryData {
                goods_price,
                offset_rate,
                true_rate,
                cart_id,
            })
        );

        // this is first item max

        /*let power = (10000 * 60 + 10000 * 40) * 30 * 15 / 1000000;

        let read_power = KpModule::kp_document_power_by_idhash(&doc_key_hash);

        println!("expected power:{}", power);
        assert_eq!(read_power.content, power);*/
    });
}

#[test]
fn kp_test_model_operate() {
    new_test_ext().execute_with(|| {
        let test_hash = H256::from_slice(&blake2_256(String::from("da038934asd1").as_bytes()));
        let test_signer_pair =
            sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

        let app_id = String::from("A01").into_bytes();
        let model_id = String::from("M01").into_bytes();
        let expert_id = String::from("E01").into_bytes();
        let commodity_name = String::from("Test").into_bytes();
        let commodity_type = String::from("Type01").into_bytes();

        let mut buf = vec![];
        buf.append(&mut (app_id.clone()));
        buf.append(&mut (model_id.clone()));

        let test_signature = test_signer_pair.sign(&buf);

        assert_ok!(KpModule::create_model(
            Origin::signed(1),
            app_id.clone(),
            model_id.clone(),
            expert_id.clone(),
            commodity_name.clone(),
            commodity_type.clone(),
            test_hash,
            test_signer_pair.public().clone().into(),
            test_signature.clone(),
            test_signer_pair.public().into(),
            test_signature,
        ));
        // asserting that the stored value is equal to what we stored
        let doc_key_hash = BlakeTwo256::hash_of(&(&app_id, &model_id));
        let read = KpModule::kp_model_data_by_idhash(&doc_key_hash);
        println!("read result:{} {}", read.owner, read.content_hash);

        assert_eq!(read.expert_id, expert_id);

        // this is first item max

        /*let power = (10000 * 60 + 10000 * 40) * 30 * 15 / 1000000;

        let read_power = KpModule::kp_document_power_by_idhash(&doc_key_hash);

        println!("expected power:{}", power);
        assert_eq!(read_power.content, power);*/
    });
}

#[test]
fn kp_account_power() {
    new_test_ext().execute_with(|| {
        // define some shared vars
        let app_id = "app01";
        let model_id = "m01";
        let document_id = "d01";
        let product_id = "p01";
        let para_issue_rate: u32 = 10;
        let self_issue_rate: u32 = 30;

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
    });
}

fn create_product_publish(
    app_id: &str,
    document_id: &str,
    model_id: &str,
    product_id: &str,
    hash_content: &str,
    para_issue_rate: u32,
    self_issue_rate: u32,
) -> dispatch::DispatchResult {
    let test_hash = H256::from_slice(&blake2_256(hash_content.as_bytes()));
    let test_signer_pair =
        sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

    let app_id_vec = app_id.as_bytes().to_vec();
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

fn create_model(
    app_id: &str,
    model_id: &str,
    expert_id: &str,
    commodity_name: &str,
    commodity_type: &str,
    hash_content: &str,
) -> dispatch::DispatchResult {
    let test_hash = H256::from_slice(&blake2_256(hash_content.as_bytes()));
    let test_signer_pair =
        sr25519::Pair::from_string(&format!("//{}", "Alice"), None).expect("valid seed");

    let app_id_vec = app_id.as_bytes().to_vec();
    let model_id_vec = model_id.as_bytes().to_vec();
    let expert_id_vec = expert_id.as_bytes().to_vec();
    let commodity_name_vec = commodity_name.as_bytes().to_vec();
    let commodity_type_vec = commodity_type.as_bytes().to_vec();

    let mut buf = vec![];
    buf.append(&mut (app_id_vec.clone()));
    buf.append(&mut (model_id_vec.clone()));

    let test_signature = test_signer_pair.sign(&buf);

    KpModule::create_model(
        Origin::signed(1),
        app_id_vec,
        model_id_vec,
        expert_id_vec,
        commodity_name_vec,
        commodity_type_vec,
        test_hash,
        test_signer_pair.public().clone().into(),
        test_signature.clone(),
        test_signer_pair.public().into(),
        test_signature,
    )
}
