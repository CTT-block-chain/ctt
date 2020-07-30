use frame_support::assert_ok;
use sp_core::H256;
use sp_io::hashing::blake2_256;

use crate::mock::*;
use crate::{DocumentSpecificData, KPProductIdentifyData, KPProductPublishData, KPProductTryData};
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

        const para_issue_rate: u32 = 32;
        const self_issue_rate: u32 = 45;

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

        const goods_price: u32 = 100;
        const ident_rate: u32 = 10;
        const ident_consistence: u32 = 23;
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

        const goods_price: u32 = 100;
        const offset_rate: u32 = 33;
        const true_rate: u32 = 21;
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
