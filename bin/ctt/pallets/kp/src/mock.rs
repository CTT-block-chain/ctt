// Creating mock runtime here

use frame_support::{impl_outer_event, impl_outer_origin, parameter_types, weights::Weight};
use sp_core::H256;
use sp_runtime::{
    testing::Header,
    traits::{BlakeTwo256, IdentityLookup},
    Perbill,
};
use sp_std::prelude::*;

use frame_system as system;

use crate::*;

mod kp {
    pub use crate::Event;
}

impl<Hash: Clone, AccountId: Clone> PartialEq for KPDocumentData<AccountId, Hash> {
    fn eq(&self, other: &Self) -> bool {
        self.document_type == other.document_type && self.document_id == other.document_id
    }
}

impl_outer_event! {
    pub enum TestEvent for Test {
        kp<T>,
        system<T>,
        pallet_balances<T>,
        members<T>,
    }
}

impl_outer_origin! {
    pub enum Origin for Test {}
}

// For testing the pallet, we construct most of a mock runtime. This means
// first constructing a configuration type (`Test`) which `impl`s each of the
// configuration traits of pallets we want to use.
#[derive(Clone, Eq, PartialEq)]
pub struct Test;
parameter_types! {
    pub const BlockHashCount: u64 = 250;
    pub const MaximumBlockWeight: Weight = 1024;
    pub const MaximumBlockLength: u32 = 2 * 1024;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);

    pub const DocumentPowerWeightAttend: u8 = 40;
    pub const DocumentPowerWeightContent: u8 = 30;
    pub const DocumentPowerWeightJudge: u8 = 30;
    pub const CommentPowerWeightCount: u8 = 35;
    pub const CommentPowerWeightCost: u8 = 40;
    pub const CommentPowerWeightPerCost: u8 = 20;
    pub const CommentPowerWeightPositive: u8 = 5;
    pub const CommentPowerWeight: u8 = 40;
    pub const DocumentPublishWeightParamsRate: u8 = 60;
    pub const DocumentPublishWeightParamsSelfRate: u8 = 40;
    pub const DocumentIdentifyWeightParamsRate: u8 = 50;
    pub const DocumentIdentifyWeightCheckRate: u8 = 50;
    pub const DocumentTryWeightBiasRate: u8 = 60;
    pub const DocumentTryWeightTrueRate: u8 = 40;
    pub const TopWeightProductPublish: u8 = 15;
    pub const TopWeightDocumentIdentify: u8 = 25;
    pub const TopWeightDocumentTry: u8 = 35;
    pub const TopWeightAccountAttend: u8 = 10;
    pub const TopWeightAccountStake: u8 = 15;
}
impl system::Trait for Test {
    type BaseCallFilter = ();
    type Origin = Origin;
    type Call = ();
    type Index = u64;
    type BlockNumber = u64;
    type Hash = H256;
    type Hashing = BlakeTwo256;
    type AccountId = u64;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Header = Header;
    type Event = TestEvent;
    type BlockHashCount = BlockHashCount;
    type MaximumBlockWeight = MaximumBlockWeight;
    type DbWeight = ();
    type BlockExecutionWeight = ();
    type ExtrinsicBaseWeight = ();
    type MaximumExtrinsicWeight = ();
    type MaximumBlockLength = MaximumBlockLength;
    type AvailableBlockRatio = AvailableBlockRatio;
    type Version = ();
    type ModuleToIndex = ();
    type AccountData = pallet_balances::AccountData<u64>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type SystemWeightInfo = ();
}

impl Trait for Test {
    type Event = TestEvent;
    type Membership = members::Module<Test>;
    type TopWeightProductPublish = TopWeightProductPublish;
    type TopWeightDocumentIdentify = TopWeightDocumentIdentify;
    type TopWeightDocumentTry = TopWeightDocumentTry;
    type TopWeightAccountAttend = TopWeightAccountAttend;
    type TopWeightAccountStake = TopWeightAccountStake;
    type DocumentPowerWeightAttend = DocumentPowerWeightAttend;
    type DocumentPowerWeightContent = DocumentPowerWeightContent;
    type DocumentPowerWeightJudge = DocumentPowerWeightJudge;
    type CommentPowerWeightCount = CommentPowerWeightCount;
    type CommentPowerWeightCost = CommentPowerWeightCost;
    type CommentPowerWeightPerCost = CommentPowerWeightPerCost;
    type CommentPowerWeightPositive = CommentPowerWeightPositive;
    type CommentPowerWeight = CommentPowerWeight;
    type DocumentPublishWeightParamsRate = DocumentPublishWeightParamsRate;
    type DocumentPublishWeightParamsSelfRate = DocumentPublishWeightParamsSelfRate;
    type DocumentIdentifyWeightParamsRate = DocumentIdentifyWeightParamsRate;
    type DocumentIdentifyWeightCheckRate = DocumentIdentifyWeightCheckRate;
    type DocumentTryWeightBiasRate = DocumentTryWeightBiasRate;
    type DocumentTryWeightTrueRate = DocumentTryWeightTrueRate;
}

impl pallet_balances::Trait for Test {
    type Balance = u64;
    type DustRemoval = ();
    type Event = TestEvent;
    type ExistentialDeposit = ();
    type AccountStore = System;
    type WeightInfo = ();
}

impl members::Trait for Test {
    type Event = TestEvent;
    type Currency = Balances;
}
pub type System = system::Module<Test>;
pub type Balances = pallet_balances::Module<Test>;

pub type KpModule = Module<Test>;

// This function basically just builds a genesis storage key/value store according to
// our desired mockup.
pub fn new_test_ext() -> sp_io::TestExternalities {
    system::GenesisConfig::default()
        .build_storage::<Test>()
        .unwrap()
        .into()
}
