#![recursion_limit = "256"]
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage, dispatch, ensure,
    traits::{Currency, Get, LockableCurrency, OnUnbalanced, Randomness, ReservableCurrency},
};
use rand_chacha::{
    rand_core::{RngCore, SeedableRng},
    ChaChaRng,
};

#[macro_use]
extern crate sp_std;

use sp_std::cmp::*;
use sp_std::ops::Add;
use sp_std::prelude::*;

/// Knowledge power pallet  with necessary imports

/// Feel free to remove or edit this file as needed.
/// If you change the name of this file, make sure to update its references in runtime/src/lib.rs
/// If you remove this file, you can remove those references

/// For more guidance on Substrate FRAME, see the example pallet
/// https://github.com/paritytech/substrate/blob/master/frame/example/src/lib.rs
use frame_system::{self as system, ensure_root, ensure_signed};
use primitives::{AuthAccountId, Membership, PowerSize};
use sp_core::sr25519;
use sp_runtime::{
    print,
    traits::{Hash, TrailingZeroInput, Verify},
    MultiSignature, RuntimeDebug,
};

pub trait PowerVote<AccountId> {
    fn account_power_ratio(_account: &AccountId) -> f64 {
        // default return 1.0
        1.0
    }
}

const FLOAT_COMPUTE_PRECISION: PowerSize = 10000;
const RATIO_DIV: f64 = 100.0;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

type CommodityPowerSet = (
    DocumentPower,
    DocumentPower,
    DocumentPower,
    PowerSize,
    PowerSize,
);

#[derive(Encode, Decode, PartialEq, Clone, RuntimeDebug)]
pub struct LeaderBoardIndex(u32);

impl Default for LeaderBoardIndex {
    fn default() -> Self {
        LeaderBoardIndex(u32::MAX)
    }
}

impl From<usize> for LeaderBoardIndex {
    fn from(v: usize) -> Self {
        LeaderBoardIndex(v as u32)
    }
}

impl LeaderBoardIndex {
    pub fn exist(&self) -> bool {
        self.0 != u32::MAX
    }

    pub fn index(&self) -> usize {
        self.0 as usize
    }
}

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, PartialEq, Clone, RuntimeDebug)]
pub enum ModelStatus {
    ENABLED = 0,
    DISABLED = 1,
}

impl Default for ModelStatus {
    fn default() -> Self {
        ModelStatus::ENABLED
    }
}

impl From<u8> for ModelStatus {
    fn from(orig: u8) -> Self {
        return match orig {
            0x0 => ModelStatus::ENABLED,
            0x1 => ModelStatus::DISABLED,
            _ => ModelStatus::ENABLED,
        };
    }
}

#[derive(Encode, Decode, PartialEq, Clone, RuntimeDebug)]
pub enum DocumentType {
    ProductPublish = 0,
    ProductIdentify,
    ProductTry,

    // this two types need special process
    ProductChoose,
    ModelCreate,

    Unknown,
}

impl Default for DocumentType {
    fn default() -> Self {
        DocumentType::ProductPublish
    }
}

impl From<u8> for DocumentType {
    fn from(orig: u8) -> Self {
        return match orig {
            0 => DocumentType::ProductPublish,
            1 => DocumentType::ProductIdentify,
            2 => DocumentType::ProductTry,
            3 => DocumentType::ProductChoose,
            4 => DocumentType::ModelCreate,
            _ => DocumentType::Unknown,
        };
    }
}

#[derive(Encode, Decode, PartialEq, Clone, RuntimeDebug)]
pub enum ModelDisputeType {
    NoneIntendNormal = 0,
    IntendNormal,
    Serious,
}

impl Default for ModelDisputeType {
    fn default() -> Self {
        ModelDisputeType::NoneIntendNormal
    }
}

#[derive(Encode, Decode, PartialEq, Clone, Copy, RuntimeDebug)]
pub enum CommentTrend {
    Positive = 0,
    Negative = 1,
    Empty = 2,
}

impl Default for CommentTrend {
    fn default() -> Self {
        CommentTrend::Empty
    }
}

impl From<u8> for CommentTrend {
    fn from(orig: u8) -> Self {
        return match orig {
            0x0 => CommentTrend::Positive,
            0x1 => CommentTrend::Negative,
            _ => CommentTrend::Empty,
        };
    }
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPProductPublishData {
    para_issue_rate: PowerSize,
    self_issue_rate: PowerSize,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPProductPublishRateMax {
    para_issue_rate: PowerSize,
    self_issue_rate: PowerSize,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPProductIdentifyData {
    goods_price: PowerSize,
    ident_rate: PowerSize,
    ident_consistence: PowerSize,
    cart_id: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPProductIdentifyRateMax {
    ident_rate: PowerSize,
    ident_consistence: PowerSize,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPProductTryData {
    goods_price: PowerSize,
    offset_rate: PowerSize,
    true_rate: PowerSize,
    cart_id: Vec<u8>,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPProductTryRateMax {
    offset_rate: PowerSize,
    true_rate: PowerSize,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPProductChooseData {
    sell_count: PowerSize,
    try_count: PowerSize,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPProductChooseDataMax {
    sell_count: PowerSize,
    try_count: PowerSize,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPModelCreateData {
    producer_count: PowerSize,
    product_count: PowerSize,
}

#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct KPModelCreateDataMax {
    producer_count: PowerSize,
    product_count: PowerSize,
}

#[derive(Encode, Decode, Clone, PartialEq, RuntimeDebug)]
pub enum DocumentSpecificData {
    ProductPublish(KPProductPublishData),
    ProductIdentify(KPProductIdentifyData),
    ProductTry(KPProductTryData),
    ProductChoose(KPProductChooseData),
    ModelCreate(KPModelCreateData),
}

impl Default for DocumentSpecificData {
    fn default() -> Self {
        DocumentSpecificData::ProductPublish(KPProductPublishData::default())
    }
}

// account comment action record
#[derive(Encode, Decode, Clone, PartialEq, Default, RuntimeDebug)]
pub struct KPCommentAccountRecord {
    count: PowerSize,
    fees: PowerSize,
    positive_count: PowerSize,
}

#[derive(Encode, Decode, Clone, PartialEq, Default, RuntimeDebug)]
pub struct CommentMaxRecord {
    max_count: PowerSize,
    max_fee: PowerSize,
    max_positive: PowerSize,

    // for document, this is the max of document's total comment cost/count
    // for account, this is the max of account's total comment fees/count
    max_unit_fee: PowerSize,
}

type KPDocumentDataOf<T> =
    KPDocumentData<<T as system::Trait>::AccountId, <T as system::Trait>::Hash>;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct KPDocumentData<AccountId, Hash> {
    app_id: u32,
    document_id: Vec<u8>,
    model_id: Vec<u8>,
    product_id: Vec<u8>,
    content_hash: Hash,
    sender: AccountId,
    owner: AuthAccountId,
    document_type: DocumentType,
    document_data: DocumentSpecificData,
    comment_count: PowerSize,
    comment_total_fee: PowerSize,
    comment_positive_count: PowerSize,
    expert_trend: CommentTrend,
    platform_trend: CommentTrend,
}

// power store
#[derive(Encode, Decode, Clone, Default, PartialEq, RuntimeDebug)]
pub struct DocumentPower {
    attend: PowerSize,
    content: PowerSize,
    judge: PowerSize,
}

impl Add for DocumentPower {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            attend: self.attend + other.attend,
            content: self.content + other.content,
            judge: self.judge + other.judge,
        }
    }
}

impl<'a, 'b> Add<&'b DocumentPower> for &'a DocumentPower {
    type Output = DocumentPower;

    fn add(self, other: &'b DocumentPower) -> DocumentPower {
        DocumentPower {
            attend: self.attend + other.attend,
            content: self.content + other.content,
            judge: self.judge + other.judge,
        }
    }
}

pub trait PowerSum {
    fn total(&self) -> PowerSize;
}

impl PowerSum for DocumentPower {
    fn total(&self) -> PowerSize {
        self.attend + self.content + self.judge
    }
}

type KPCommentDataOf<T> =
    KPCommentData<<T as system::Trait>::AccountId, <T as system::Trait>::Hash>;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct KPCommentData<AccountId, Hash> {
    app_id: u32,
    document_id: Vec<u8>,
    comment_id: Vec<u8>,
    comment_hash: Hash,
    comment_fee: PowerSize,
    comment_trend: u8,
    sender: AccountId,
    owner: AuthAccountId,
}

type KPModelDataOf<T> = KPModelData<<T as system::Trait>::AccountId, <T as system::Trait>::Hash>;
#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct KPModelData<AccountId, Hash> {
    app_id: u32,
    model_id: Vec<u8>,
    expert_id: Vec<u8>,
    status: ModelStatus,
    commodity_name: Vec<u8>,
    commodity_type: u32,
    content_hash: Hash,
    sender: AccountId,
    owner: AuthAccountId,
}

#[derive(Encode, Decode, Clone, Default, Eq, RuntimeDebug)]
pub struct CommodityTypeData {
    type_id: u32,
    type_desc: Vec<u8>,
}

impl PartialEq for CommodityTypeData {
    fn eq(&self, other: &Self) -> bool {
        self.type_id == other.type_id
    }
}

impl Ord for CommodityTypeData {
    fn cmp(&self, other: &Self) -> Ordering {
        self.type_id.cmp(&other.type_id)
    }
}

impl PartialOrd for CommodityTypeData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Encode, Decode, Clone, RuntimeDebug)]
pub struct AppFinancedData<T: Trait> {
    amount: BalanceOf<T>,
    exchange_rate: BalanceOf<T>,
}

impl<T: Trait> Default for AppFinancedData<T> {
    fn default() -> Self {
        AppFinancedData::<T> {
            amount: 0.into(),
            exchange_rate: 0.into(),
        }
    }
}

#[derive(Encode, Decode, Default, Clone, Eq, RuntimeDebug)]
pub struct CommodityLeaderBoardData<T: Trait> {
    cart_id: Vec<u8>,
    cart_id_hash: T::Hash,
    power: PowerSize,
}

impl<T: Trait> PartialEq for CommodityLeaderBoardData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.power == other.power
    }
}

impl<T: Trait> Ord for CommodityLeaderBoardData<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.power.cmp(&other.power)
    }
}

impl<T: Trait> PartialOrd for CommodityLeaderBoardData<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Encode, Decode, Clone, Eq, Default, RuntimeDebug)]
pub struct CommentWeightData<T: Trait> {
    account: T::AccountId,
    position: u64,
    cash_cost: PowerSize,
}

impl<T: Trait> PartialEq for CommentWeightData<T> {
    fn eq(&self, other: &Self) -> bool {
        self.account == other.account
    }
}

impl<T: Trait> Ord for CommentWeightData<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.position.cmp(&other.position)
    }
}

impl<T: Trait> PartialOrd for CommentWeightData<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/*
type KnowledgePowerDataOf<T> = KnowledgePowerData<<T as system::Trait>::AccountId>;

#[derive(Encode, Decode, Clone, Default, RuntimeDebug)]
pub struct KnowledgePowerData<AccountId> {
    app_id: Vec<u8>,
    knowledge_id: Vec<u8>,
    owner: AccountId,
    power: u32,
    // A: knowledge owner total profit
    owner_profit: u32,
    // B: comment total count
    comment_total_count: u32,
    // C: total user number of attending comment action
    comment_total_user: u32,
    // D: total cost of comments
    comment_total_cost: u32,
    // E: max cost of comment
    comment_max_cost: u32,
    // F: comments which repeated users count, for example: AABBBCD, 2 + 3
    comment_repeat_user_count: u32,
    // G: comment cost increase count
    comment_cost_increase_count: u32,
    // H: comment count of (user = knowledge owner)
    comment_self_count: u32,
}

/// our power compute algo is:
/// p = (comment_total_user * comment_total_cost) * (1 + comment_cost_increase_count / comment_total_count)
/// 	/ (owner_profit * (comment_self_count / comment_total_count + comment_repeat_user_count / comment_total_count) )
/// 	* comment_max_cost / comment_cost_increase_count
/// 	* (extra_compute_param / 100)
///
/// With simple symbol:
/// p = ((C * D) * (1 + G / B) / (A * (H / B + F / B))) * (E / G) * (ep / 100)
/// Simplified to:
/// p = ((C * D * E * (B + G)) / (A * G * (H + F)) * (ep / 100)
fn power_update<T: system::Trait>(power_data: &KnowledgePowerData<T::AccountId>, ep: u32) -> u32 {
    match power_data {
        KnowledgePowerData {
            app_id: _,
            knowledge_id: _,
            owner: _,
            power: _,
            owner_profit: a,
            comment_total_count: b,
            comment_total_user: c,
            comment_total_cost: d,
            comment_max_cost: e,
            comment_repeat_user_count: f,
            comment_cost_increase_count: g,
            comment_self_count: h,
        } => {
            if *a == 0 || *g == 0 {
                print("Power compute 0, because has 0 value in den !");
                return 0;
            }

            // TODO: overflow check
            // c * d * e * (b + g) / (a * g * (h + f)) * (ep / 100)
            let step1 = c * d * e * (b + g);
            let mut step2 = a * g;
            if h + f > 0 {
                step2 *= h + f;
            }

            let result: u32 = step1 * ep / step2 / 100;
            result
        }
    }
}*/

/// The pallet's configuration trait.
pub trait Trait: system::Trait {
    // Add other types and constants required to configure this pallet.
    /// Membership control
    type Membership: Membership<Self::AccountId, Self::Hash>;

    /// Currency type for this module.
    type Currency: ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

    /// Handler for the unbalanced reduction when slashing a model create deposit.
    type Slash: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// Something that provides randomness in the runtime.
    type Randomness: Randomness<Self::Hash>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;

    /// 5 dimensions weight config
    type TopWeightProductPublish: Get<u8>;
    type TopWeightDocumentIdentify: Get<u8>;
    type TopWeightDocumentTry: Get<u8>;
    type TopWeightAccountAttend: Get<u8>;
    type TopWeightAccountStake: Get<u8>;

    /// Document Power attend weight
    type DocumentPowerWeightAttend: Get<u8>;

    /// Document Power content weight
    type DocumentPowerWeightContent: Get<u8>;

    /// Document Power judge weight
    type DocumentPowerWeightJudge: Get<u8>;

    /// Comment Power count weight
    type CommentPowerWeightCount: Get<u8>;

    /// Comment Power cost weight
    type CommentPowerWeightCost: Get<u8>;

    /// Comment Power cost per uint weight
    type CommentPowerWeightPerCost: Get<u8>;

    /// Comment Power positive weight
    type CommentPowerWeightPositive: Get<u8>;

    type CommentPowerWeight: Get<u8>;

    /// Document Publish content weight
    type DocumentPublishWeightParamsRate: Get<u8>;
    type DocumentPublishWeightParamsSelfRate: Get<u8>;

    /// Document Identify content weight
    type DocumentIdentifyWeightParamsRate: Get<u8>;
    type DocumentIdentifyWeightCheckRate: Get<u8>;

    /// Document Try content weight
    type DocumentTryWeightBiasRate: Get<u8>;
    type DocumentTryWeightTrueRate: Get<u8>;

    /// Below for Choose & Model special documents
    /// Document Choose content weight
    type DocumentChooseWeightSellCount: Get<u8>;
    type DocumentChooseWeightTryCount: Get<u8>;

    /// Document Model content weight
    type DocumentModelWeightProducerCount: Get<u8>;
    type DocumentModelWeightProductCount: Get<u8>;

    /// Document Choose & Model Power attend weight
    type DocumentCMPowerWeightAttend: Get<u8>;

    /// Document Choose & Model Power content weight
    type DocumentCMPowerWeightContent: Get<u8>;

    /// Document Choose & Model Power judge weight
    type DocumentCMPowerWeightJudge: Get<u8>;

    /// Comment Power count weight
    type CommentCMPowerWeightCount: Get<u8>;

    /// Comment Power cost weight
    type CommentCMPowerWeightCost: Get<u8>;

    /// Comment Power cost per uint weight
    type CommentCMPowerWeightPerCost: Get<u8>;

    /// Comment Power positive weight
    type CommentCMPowerWeightPositive: Get<u8>;

    type CMPowerAccountAttend: Get<u8>;

    type ModelCreateDeposit: Get<BalanceOf<Self>>;

    /// App financed purpose minimal exchange rate, aka 1 RMB exchange how many KPT
    type KptExchangeMinRate: Get<BalanceOf<Self>>;

    type AppLeaderBoardInterval: Get<Self::BlockNumber>;

    type AppLeaderBoardMaxPos: Get<u32>;
}

// This pallet's storage items.
decl_storage! {
    // It is important to update your storage name so that your pallet's
    // storage items are isolated from other pallets.
    trait Store for Module<T: Trait> as Kp {
        // Trusted application server account
        AuthServers get(fn auth_servers) config() : Vec<T::AccountId>;

        // App id ranges according type string
        AppIdRange get(fn app_id_range) config():
            map hasher(twox_64_concat) Vec<u8> => u32;

        // (AppId, ModelId) -> KPModelData
        KPModelDataByIdHash get(fn kp_model_data_by_idhash):
            map hasher(twox_64_concat) T::Hash => KPModelDataOf<T>;

        // (AppId, ModelId) -> BalanceOf<T>  deposit value of create model
        KPModelDepositMap get(fn kp_model_):
            map hasher(twox_64_concat) T::Hash => BalanceOf<T>;

        // (AppId, AuthAccountId) -> KPCommentAccountRecord
        KPCommentAccountRecordMap get(fn kp_comment_account_record_map):
            map hasher(twox_64_concat) T::Hash => KPCommentAccountRecord;

        // AuthAccountId -> PowerSize max goods_price
        KPAccountMaxPurchaseByIdHash get(fn kp_account_max_purchase_by_idhash):
            map hasher(twox_64_concat) AuthAccountId => PowerSize;

        // (AppId, CartId) -> PowerSize user computed product identify/try power map
        // (Publish, Identify, Try, OwnerAction, OwnerEconomic)
        KPPurchasePowerByIdHash get(fn kp_purchase_power_by_idhash):
            map hasher(twox_64_concat) T::Hash => (DocumentPower, DocumentPower, DocumentPower, PowerSize, PowerSize);

        // Slash commodity power black list (AppId, CartId) -> bool
        KPPurchaseBlackList get(fn kp_purchase_black_list):
            map hasher(twox_64_concat) T::Hash => bool;

        // (AppId, DocumentId) -> PowerSize misc document power map (currently for product choose and model create)
        KPMiscDocumentPowerByIdHash get(fn kp_misc_document_power_by_idhash):
            map hasher(twox_64_concat) T::Hash => PowerSize;

        // (AppId, DocumentId) -> KPDocumentData
        KPDocumentDataByIdHash get(fn kp_document_data_by_idhash):
            map hasher(twox_64_concat) T::Hash => KPDocumentDataOf<T>;

        // (AppId, DocumentId) -> document power
        KPDocumentPowerByIdHash get(fn kp_document_power_by_idhash):
            map hasher(twox_64_concat) T::Hash => DocumentPower;

        // (AppId, ProductId) -> DocumentId document index map
        KPDocumentProductIndexByIdHash get(fn kp_document_product_index_by_idhash):
            map hasher(twox_64_concat) T::Hash => Vec<u8>;

        // (AppId, CartId) -> Vec<u8> cartid -> product identify document id
        KPCartProductIdentifyIndexByIdHash get(fn kp_cart_product_identify_index_by_idhash):
            map hasher(twox_64_concat) T::Hash => Vec<u8>;

        // (AppId, CartId) -> Vec<u8> cartid -> product try document id
        KPCartProductTryIndexByIdHash get(fn kp_cart_product_try_index_by_idhash):
            map hasher(twox_64_concat) T::Hash => Vec<u8>;

        // (AppId, CommentId) -> KnowledgeCommentData
        KPCommentDataByIdHash get(fn kp_comment_data_by_idhash):
            map hasher(twox_64_concat) T::Hash => KPCommentDataOf<T>;

        // global total knowledge power
        TotalPower get(fn total_power): PowerSize;

        // miner power table
        MinerPowerByAccount get(fn miner_power_by_account):
            map hasher(blake2_128_concat) T::AccountId => PowerSize;

        // miner documents power (accumulation) (app_id account_id) -> DocumentPower
        MinerDocumentsAccumulationPower get(fn miner_documents_accumulation_power):
            map hasher(twox_64_concat) T::Hash => DocumentPower;

        // account attend power (AccountId, AppId) -> PowerSize
        AccountAttendPowerMap get(fn account_attend_power_map):
            map hasher(blake2_128_concat) T::Hash => PowerSize;

        // global power compute related parameters:
        // AppId -> single document's max comment count
        CommentMaxInfoPerDocMap get(fn comment_max_info_per_doc_map):
            map hasher(twox_64_concat) u32 => CommentMaxRecord;

        // AppId -> single account's max comment count
        CommentMaxInfoPerAccountMap get(fn comment_max_info_per_account_map):
            map hasher(twox_64_concat) u32 => CommentMaxRecord;

        // AppId -> single account's max goods_price
        MaxGoodsPricePerAccountMap get(fn max_goods_price_per_account_map):
            map hasher(twox_64_concat) u32 => PowerSize;

        // AppId -> document publish params max
        DocumentPublishMaxParams get(fn document_publish_max_params):
            map hasher(twox_64_concat) u32 => KPProductPublishRateMax;

        DocumentIdentifyMaxParams get(fn document_identify_max_params):
            map hasher(twox_64_concat) u32 => KPProductIdentifyRateMax;

        DocumentTryMaxParams get(fn document_try_max_params):
            map hasher(twox_64_concat) u32 => KPProductTryRateMax;

        DocumentChooseMaxParams get(fn document_choose_max_params):
            map hasher(twox_64_concat) u32 => KPProductChooseDataMax;

        DocumentModelCreateMaxParams get(fn document_model_create_max_params):
            map hasher(twox_64_concat) u32 => KPModelCreateDataMax;

        CommodityTypeSets get(fn commodity_type_sets): Vec<CommodityTypeData>;

        // commodity_type_id => type desc map
        CommodityTypeMap get(fn commodity_type_map):
            map hasher(twox_64_concat) u32 => Vec<u8>;

        // app_id & commodity_type_id => true/false
        ModelFirstTypeBenefitRecord get(fn model_first_type_benefit_record):
            map hasher(twox_64_concat) T::Hash => bool;

        // app id => u32
        AppModelTotalConfig get(fn app_model_total_config):
            map hasher(twox_64_concat) u32 => u32;

        // app id => u32
        AppModelCount get(fn app_model_count):
            map hasher(twox_64_concat) u32 => u32;

        // model year incoming double map, main key is year (u32), sub key is hash of AppId & ModelId
        ModelYearIncome get(fn model_year_income):
            double_map hasher(twox_64_concat) u32, hasher(twox_64_concat) T::Hash => u64;

        AppYearIncomeTotal get(fn app_year_income_total):
            map hasher(twox_64_concat) u32 => u64;

        // App financed record
        AppFinancedRecord get(fn app_financed_record):
            map hasher(twox_64_concat) u32 => AppFinancedData<T>;

        // App commodity(cart_id) count AppId -> u32
        AppCommodityCount get(fn app_commodity_count):
            map hasher(twox_64_concat) u32 => u32;

        // App model commodity(cart_id) count (AppId, ModelId) -> u32
        AppModelCommodityCount get(fn app_model_commodity_count):
            map hasher(twox_64_concat) T::Hash => u32;

        // Model commodity realtime power leader boards (AppId, ModelId) => Set of board data
        AppModelCommodityLeaderBoards get(fn app_model_commodity_leader_boards):
            map hasher(twox_64_concat) T::Hash => Vec<CommodityLeaderBoardData<T>>;

        // If a commodity(cart id) enter leader board, we use a map set to record it for
        // fast check if board contains it when doing board update
        // double map, group key: (AppId, ModelId) hash, sub key: cart_id
        LeaderBoardCommoditySet get(fn leader_board_commodity_set):
            double_map hasher(twox_64_concat) T::Hash, hasher(twox_64_concat) Vec<u8> => ();

        // Leader board history records (AppId, ModelId, BlockNumber) => (Vec<CommodityLeaderBoardData>, Vec<T::AccountId>)
        AppLeaderBoardRcord get(fn app_leader_board_record):
            map hasher(twox_64_concat) T::Hash => (Vec<CommodityLeaderBoardData<T>>, Vec<T::AccountId>);

        // Leader board last record (AppId, ModelId) -> BlockNumber
        AppLeaderBoardLastTime get(fn app_leader_board_last_time):
            map hasher(twox_64_concat) T::Hash => T::BlockNumber;

        // Document comment order pool (AppId, DocumentId) -> Vec<CommentWeightData>
        DocumentCommentsAccountPool get(fn document_comments_account_pool):
            map hasher(twox_64_concat) T::Hash => Vec<CommentWeightData<T>>;
    }
}

// The pallet's events
decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        BlockNumber = <T as system::Trait>::BlockNumber,
    {
        /// Just a dummy event.
        /// Event `Something` is declared with a parameter of the type `u32` and `AccountId`
        /// To emit this event, we call the deposit function, from our runtime functions
        // SomethingStored(u32, AccountId),
        KnowledgeCreated(AccountId),
        CommentCreated(AccountId),
        ModelCreated(AccountId),
        ModelDisabled(AccountId),
        CommodityTypeCreated(u32),
        AppModelTotal(u32),
        ModelYearIncome(AccountId),
        PowerSlashed(AccountId),
        AppAdded(u32),
        AppFinanced(u32),
        LeaderBoardsCreated(BlockNumber),
        ModelDisputed(AccountId),
    }
);

// The pallet's errors
decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Some action needs to check specified account has enough balance to pay for gas fee.
        BalanceNotEnough,
        AddOverflow,
        DocumentAlreadyExisted,
        ProductAlreadyExisted,
        CommentAlreadyExisted,
        ModelAlreadyExisted,
        ModelTypeInvalid,
        ModelNotFound,
        CommodityTypeExisted,
        ModelOverSizeLimit,
        NotAppAdmin,
        ModelYearIncomeAlreadyExisted,
        CommentNotFound,
        DocumentNotFound,
        ProductNotFound,
        AppTypeInvalid,
        ReturnRateInvalid,
        AppIdInvalid,
        AppAlreadyFinanced,
        AppFinancedExchangeRateTooLow,
        DocumentIdentifyAlreadyExisted,
        DocumentTryAlreadyExisted,
        LeaderBoardCreateNotPermit,
    }
}

// The pallet's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing errors
        // this includes information about your errors in the node's metadata.
        // it is needed only if you are using errors in your pallet
        type Error = Error<T>;

        // Initializing events
        // this is needed only if you are using events in your pallet
        fn deposit_event() = default;

        const TopWeightProductPublish: u8 = T::TopWeightProductPublish::get();
        const TopWeightDocumentIdentify: u8 = T::TopWeightDocumentIdentify::get();
        const TopWeightDocumentTry: u8 = T::TopWeightDocumentTry::get();
        const TopWeightAccountAttend: u8 = T::TopWeightAccountAttend::get();
        const TopWeightAccountStake: u8 = T::TopWeightAccountStake::get();

        const DocumentPowerWeightAttend: u8 = T::DocumentPowerWeightAttend::get();
        const DocumentPowerWeightContent: u8 = T::DocumentPowerWeightContent::get();
        const DocumentPowerWeightJudge: u8 = T::DocumentPowerWeightJudge::get();

        const CommentPowerWeightCount: u8 = T::CommentPowerWeightCount::get();
        const CommentPowerWeightCost: u8 = T::CommentPowerWeightCost::get();
        const CommentPowerWeightPerCost: u8 = T::CommentPowerWeightPerCost::get();
        const CommentPowerWeightPositive: u8 = T::CommentPowerWeightPositive::get();
        const CommentPowerWeight: u8 = T::CommentPowerWeight::get();

        const DocumentPublishWeightParamsRate: u8 = T::DocumentPublishWeightParamsRate::get();
        const DocumentPublishWeightParamsSelfRate: u8 = T::DocumentPublishWeightParamsSelfRate::get();

        const DocumentIdentifyWeightParamsRate: u8 = T::DocumentIdentifyWeightParamsRate::get();
        const DocumentIdentifyWeightCheckRate: u8 = T::DocumentIdentifyWeightCheckRate::get();

        const DocumentTryWeightBiasRate: u8 = T::DocumentTryWeightBiasRate::get();
        const DocumentTryWeightTrueRate: u8 = T::DocumentTryWeightTrueRate::get();

        // CM parameters
        const DocumentChooseWeightSellCount: u8 = T::DocumentChooseWeightSellCount::get();
        const DocumentChooseWeightTryCount: u8 = T::DocumentChooseWeightTryCount::get();
        const DocumentModelWeightProducerCount: u8 = T::DocumentModelWeightProducerCount::get();
        const DocumentModelWeightProductCount: u8 = T::DocumentModelWeightProductCount::get();
        const DocumentCMPowerWeightAttend: u8 = T::DocumentCMPowerWeightAttend::get();
        const DocumentCMPowerWeightContent: u8 = T::DocumentCMPowerWeightContent::get();
        const DocumentCMPowerWeightJudge: u8 = T::DocumentCMPowerWeightJudge::get();
        const CommentCMPowerWeightCount: u8 = T::CommentCMPowerWeightCount::get();
        const CommentCMPowerWeightCost: u8 = T::CommentCMPowerWeightCost::get();
        const CommentCMPowerWeightPerCost: u8 = T::CommentCMPowerWeightPerCost::get();
        const CommentCMPowerWeightPositive: u8 = T::CommentCMPowerWeightPositive::get();
        const CMPowerAccountAttend: u8 = T::CMPowerAccountAttend::get();

        const ModelCreateDeposit: BalanceOf<T> = T::ModelCreateDeposit::get();
        const KptExchangeMinRate: BalanceOf<T> = T::KptExchangeMinRate::get();

        #[weight = 0]
        pub fn create_model(origin,
            app_id: u32,
            model_id: Vec<u8>,
            expert_id: Vec<u8>,
            commodity_name: Vec<u8>,
            commodity_type: u32,
            content_hash: T::Hash,

            app_user_account: AuthAccountId,
            app_user_sign: sr25519::Signature,

            auth_server: AuthAccountId,
            auth_sign: sr25519::Signature

            )-> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            // TODO: verify 2 signature

            let key = T::Hashing::hash_of(&(app_id, &model_id));
            ensure!(!<KPModelDataByIdHash<T>>::contains_key(&key), Error::<T>::ModelAlreadyExisted);

            print(commodity_type);

            // check if valid commodity_type
            ensure!(CommodityTypeMap::contains_key(commodity_type),  Error::<T>::ModelTypeInvalid);

            let count = <AppModelCount>::get(app_id);
            ensure!(count < <AppModelTotalConfig>::get(app_id), Error::<T>::ModelOverSizeLimit);

            print("checking deposit");
            // deposit
            let user_account = Self::convert_account(&app_user_account);
            let value = T::ModelCreateDeposit::get();
            T::Currency::reserve(&user_account, value)?;
            <KPModelDepositMap<T>>::insert(&key, value);

            let model = KPModelData {
                app_id,
                model_id,
                expert_id,
                status: ModelStatus::ENABLED,
                commodity_name,
                commodity_type,
                content_hash,
                sender: who.clone(),
                owner: app_user_account,
            };

            <KPModelDataByIdHash<T>>::insert(&key, &model);
            <AppModelCount>::insert(app_id, count + 1);

            let type_key = T::Hashing::hash_of(&(app_id, commodity_type));
            let should_transfer = !<ModelFirstTypeBenefitRecord<T>>::contains_key(&type_key);
            T::Membership::set_model_creator(&key, &(Self::convert_account(&model.owner)), &who, should_transfer);
            if should_transfer {
                <ModelFirstTypeBenefitRecord<T>>::insert(&type_key, true);
            }

            Self::deposit_event(RawEvent::ModelCreated(who));
            Ok(())
        }

        #[weight = 0]
        pub fn disable_model(origin, app_id: Vec<u8>, model_id: Vec<u8>,
            app_user_account: AuthAccountId,
            app_user_sign: sr25519::Signature,

            auth_server: AuthAccountId,
            auth_sign: sr25519::Signature
            )-> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;

            let key = T::Hashing::hash_of(&(app_id, &model_id));
            ensure!(<KPModelDataByIdHash<T>>::contains_key(&key), Error::<T>::ModelNotFound);

            <KPModelDataByIdHash<T>>::mutate(&key, |model| {
                model.status = ModelStatus::DISABLED;
            });

            // TODO: delay return of deposit money

            Self::deposit_event(RawEvent::ModelDisabled(who));
            Ok(())
        }

        #[weight = 0]
        pub fn create_product_publish_document(origin,
            app_id: u32,
            document_id: Vec<u8>,
            model_id: Vec<u8>,
            product_id: Vec<u8>,
            content_hash: T::Hash,

            document_power_data: KPProductPublishData,

            app_user_account: AuthAccountId,
            app_user_sign: sr25519::Signature,

            auth_server: AuthAccountId,
            auth_sign: sr25519::Signature) -> dispatch::DispatchResult {

            // Check it was signed and get the signer. See also: ensure_root and ensure_none
            let who = ensure_signed(origin)?;
            // check if document_id is existed already.
            /*let mut doc_key_buf = vec![];
            doc_key_buf.append(&mut(app_id.clone()));
            doc_key_buf.append(&mut(document_id.clone()));
            let doc_key = H160::from(doc_key_buf);*/

            let doc_key_hash = T::Hashing::hash_of(&(app_id, &document_id));

            ensure!(!<KPDocumentDataByIdHash<T>>::contains_key(&doc_key_hash), Error::<T>::DocumentAlreadyExisted);

            // extract percent rates data

            // Validation checks:
            // check if product_id already existed
            let product_key_hash = T::Hashing::hash_of(&(app_id, &product_id));
            ensure!(!<KPDocumentProductIndexByIdHash<T>>::contains_key(&product_key_hash), Error::<T>::ProductAlreadyExisted);

            // check if model exist
            let model_key = T::Hashing::hash_of(&(app_id, &model_id));
            ensure!(<KPModelDataByIdHash<T>>::contains_key(&model_key), Error::<T>::ModelNotFound);

            // TODO: 2 sign verification
            // construct verification u8 array:
            /*let mut buf = vec![];
            buf.append(&mut(app_id.clone()));
            buf.append(&mut(document_id.clone()));
            buf.append(&mut vec![knowledge_type, extra_compute_param]);

            // auth sign check with auth_server & auth_sign
            ensure!(Self::auth_server_verify(auth_server, auth_sign, &buf), "auth server signature verification fail");*/

            // TODO: validate data

            let doc = KPDocumentData {
                sender: who.clone(),
                owner: app_user_account.clone(),
                document_type: DocumentType::ProductPublish,
                app_id,
                document_id: document_id.clone(),
                model_id,
                product_id: product_id.clone(),
                content_hash,
                document_data: DocumentSpecificData::ProductPublish(document_power_data),
                ..Default::default()
            };

            Self::process_document_content_power(&doc);
            Self::process_commodity_power(&doc);

            // create document record
            <KPDocumentDataByIdHash<T>>::insert(&doc_key_hash, &doc);

            // create product id -> document id record
            <KPDocumentProductIndexByIdHash<T>>::insert(&product_key_hash, &document_id);

            Self::deposit_event(RawEvent::KnowledgeCreated(who));
            Ok(())
        }

        #[weight = 0]
        pub fn create_product_identify_document(origin,
            app_id: u32,
            document_id: Vec<u8>,
            _model_id: Vec<u8>,
            product_id: Vec<u8>,
            content_hash: T::Hash,

            document_power_data: KPProductIdentifyData,

            app_user_account: AuthAccountId,
            app_user_sign: sr25519::Signature,

            auth_server: AuthAccountId,
            auth_sign: sr25519::Signature) -> dispatch::DispatchResult {

            let who = ensure_signed(origin)?;

            let doc_key_hash = T::Hashing::hash_of(&(app_id, &document_id));
            ensure!(!<KPDocumentDataByIdHash<T>>::contains_key(&doc_key_hash), Error::<T>::DocumentAlreadyExisted);

            let product_key_hash = T::Hashing::hash_of(&(app_id, &product_id));
            ensure!(<KPDocumentProductIndexByIdHash<T>>::contains_key(&product_key_hash), Error::<T>::ProductNotFound);

            let cart_id = document_power_data.cart_id.clone();

            let key = T::Hashing::hash_of(&(app_id, &cart_id));
            ensure!(!<KPCartProductIdentifyIndexByIdHash<T>>::contains_key(&key), Error::<T>::DocumentIdentifyAlreadyExisted);

            let model_id = Self::get_model_id_from_product(app_id, &product_id).unwrap_or_default();
            let cart_id = document_power_data.cart_id.clone();

            // create doc
            let doc = KPDocumentData {
                sender: who.clone(),
                owner: app_user_account.clone(),
                document_type: DocumentType::ProductIdentify,
                app_id,
                document_id: document_id.clone(),
                model_id,
                product_id,
                content_hash,
                document_data: DocumentSpecificData::ProductIdentify(document_power_data),
                ..Default::default()
            };

            // process content power
            Self::process_document_content_power(&doc);
            Self::process_commodity_power(&doc);

            // create cartid -> product identify document id record
            <KPCartProductIdentifyIndexByIdHash<T>>::insert(&key, &document_id);

            // create document record
            <KPDocumentDataByIdHash<T>>::insert(&doc_key_hash, &doc);

            Self::increase_commodity_count(
                app_id,
                &doc.model_id,
                &cart_id,
                DocumentType::ProductIdentify,
            );

            Self::deposit_event(RawEvent::KnowledgeCreated(who));
            Ok(())
        }

        #[weight = 0]
        pub fn create_product_try_document(origin,
            app_id: u32,
            document_id: Vec<u8>,
            _model_id: Vec<u8>,
            product_id: Vec<u8>,
            content_hash: T::Hash,

            document_power_data: KPProductTryData,

            app_user_account: AuthAccountId,
            app_user_sign: sr25519::Signature,

            auth_server: AuthAccountId,
            auth_sign: sr25519::Signature) -> dispatch::DispatchResult {

            let who = ensure_signed(origin)?;

            let doc_key_hash = T::Hashing::hash_of(&(app_id, &document_id));
            ensure!(!<KPDocumentDataByIdHash<T>>::contains_key(&doc_key_hash), Error::<T>::DocumentAlreadyExisted);

            let product_key_hash = T::Hashing::hash_of(&(app_id, &product_id));
            ensure!(<KPDocumentProductIndexByIdHash<T>>::contains_key(&product_key_hash), Error::<T>::ProductNotFound);

            let cart_id = document_power_data.cart_id.clone();
            let key = T::Hashing::hash_of(&(app_id, &cart_id));
            ensure!(!<KPCartProductTryIndexByIdHash<T>>::contains_key(&key), Error::<T>::DocumentTryAlreadyExisted);

            let model_id = Self::get_model_id_from_product(app_id, &product_id).unwrap_or_default();
            let cart_id = document_power_data.cart_id.clone();

            // create doc
            let doc = KPDocumentData {
                sender: who.clone(),
                owner: app_user_account.clone(),
                document_type: DocumentType::ProductTry,
                app_id,
                document_id: document_id.clone(),
                model_id,
                product_id,
                content_hash,
                document_data: DocumentSpecificData::ProductTry(document_power_data),
                ..Default::default()
            };

            // process content power
            Self::process_document_content_power(&doc);
            Self::process_commodity_power(&doc);

            // create cartid -> product identify document id record

            <KPCartProductTryIndexByIdHash<T>>::insert(&key, &document_id);

            // create document record
            <KPDocumentDataByIdHash<T>>::insert(&doc_key_hash, &doc);

            Self::increase_commodity_count(
                app_id,
                &doc.model_id,
                &cart_id,
                DocumentType::ProductTry,
            );

            Self::deposit_event(RawEvent::KnowledgeCreated(who));
            Ok(())
        }

        #[weight = 0]
        pub fn create_comment(origin,
            app_id: u32,
            comment_id: Vec<u8>,
            document_id: Vec<u8>,

            comment_hash: T::Hash,
            comment_fee: PowerSize,
            comment_trend: u8,

            app_user_account: AuthAccountId,
            app_user_sign: sr25519::Signature,

            auth_server: AuthAccountId,
            auth_sign: sr25519::Signature) -> dispatch::DispatchResult {

            let who = ensure_signed(origin)?;

            // TODO: 2 sign verification

            // TODO: check platform & expert member role

            // make sure this comment not exist
            let key = T::Hashing::hash_of(&(app_id, &comment_id));
            ensure!(!<KPCommentDataByIdHash<T>>::contains_key(&key), Error::<T>::CommentAlreadyExisted);

            let doc_key_hash = T::Hashing::hash_of(&(app_id, &document_id));

            let comment = KPCommentData {
                sender: who.clone(),
                owner: app_user_account.clone(),
                app_id,
                document_id: document_id.clone(),
                comment_id: comment_id.clone(),
                comment_fee,
                comment_trend,
                comment_hash,
            };

            Self::process_comment_power(&comment);

            // read out related document, trigger account power update
            let doc = Self::kp_document_data_by_idhash(&doc_key_hash);
            Self::process_commodity_power(&doc);

            // create comment record
            <KPCommentDataByIdHash<T>>::insert(&key, &comment);

            Self::deposit_event(RawEvent::CommentCreated(who));
            Ok(())
        }

        #[weight = 0]
        pub fn create_commodity_type(origin, type_id: u32, type_desc: Vec<u8>) -> dispatch::DispatchResult {
            ensure_root(origin)?;
            ensure!(!<CommodityTypeMap>::contains_key(type_id), Error::<T>::CommodityTypeExisted);

            let mut types = CommodityTypeSets::get();

            let type_data = CommodityTypeData {
                type_id,
                type_desc: type_desc.clone()
            };

            match types.binary_search(&type_data) {
                Ok(_) => Err(Error::<T>::CommodityTypeExisted.into()),
                Err(index) => {
                    types.insert(index, type_data);
                    CommodityTypeSets::put(types);

                    // insert into CommodityTypeMap
                    <CommodityTypeMap>::insert(type_id, type_desc);

                    Self::deposit_event(RawEvent::CommodityTypeCreated(type_id));
                    Ok(())
                }
            }
        }

        #[weight = 0]
        pub fn set_app_model_total(origin, app_id: u32, total: u32) -> dispatch::DispatchResult {
            ensure_root(origin)?;

            <AppModelTotalConfig>::insert(app_id, total);

            Self::deposit_event(RawEvent::AppModelTotal(total));
            Ok(())
        }

        #[weight = 0]
        pub fn set_model_year_income(origin, year: u32, app_id: u32, model_id: Vec<u8>, income: u64) -> dispatch::DispatchResult {
            let who = ensure_signed(origin)?;
            // check if who is app admin
            ensure!(T::Membership::is_app_admin(&who, app_id), Error::<T>::NotAppAdmin);

            let subkey = T::Hashing::hash_of(&(app_id, &model_id));

            // check if it is existed already
            ensure!(!<ModelYearIncome<T>>::contains_key(year, &subkey), Error::<T>::ModelYearIncomeAlreadyExisted);

            // add this model income to year total
            let result = match <AppYearIncomeTotal>::get(year).checked_add(income) {
                Some(r) => r,
                None => return Err(<Error<T>>::AddOverflow.into()),
            };
            <AppYearIncomeTotal>::insert(year, result);
            <ModelYearIncome<T>>::insert(year, &subkey, income);

            Self::deposit_event(RawEvent::ModelYearIncome(who));
            Ok(())
        }

        #[weight = 0]
        pub fn create_product_choose_document(origin,
            app_id: u32,
            document_id: Vec<u8>,
            model_id: Vec<u8>,
            product_id: Vec<u8>,
            content_hash: T::Hash,

            document_power_data: KPProductChooseData,

            app_user_account: AuthAccountId,
            app_user_sign: sr25519::Signature,

            auth_server: AuthAccountId,
            auth_sign: sr25519::Signature) -> dispatch::DispatchResult {

            let who = ensure_signed(origin)?;

            let doc_key_hash = T::Hashing::hash_of(&(app_id, &document_id));

            ensure!(!<KPDocumentDataByIdHash<T>>::contains_key(&doc_key_hash), Error::<T>::DocumentAlreadyExisted);

            // create doc
            let doc = KPDocumentData {
                sender: who.clone(),
                owner: app_user_account.clone(),
                document_type: DocumentType::ProductChoose,
                app_id,
                document_id: document_id.clone(),
                model_id,
                product_id,
                content_hash,
                document_data: DocumentSpecificData::ProductChoose(document_power_data),
                ..Default::default()
            };

            // process content power
            Self::process_document_content_power(&doc);
            Self::process_commodity_power(&doc);

            // create document record
            <KPDocumentDataByIdHash<T>>::insert(&doc_key_hash, &doc);

            Self::deposit_event(RawEvent::KnowledgeCreated(who));
            Ok(())
        }

        #[weight = 0]
        pub fn create_model_create_document(origin,
            app_id: u32,
            document_id: Vec<u8>,
            model_id: Vec<u8>,
            product_id: Vec<u8>,
            content_hash: T::Hash,

            document_power_data: KPModelCreateData,

            app_user_account: AuthAccountId,
            app_user_sign: sr25519::Signature,

            auth_server: AuthAccountId,
            auth_sign: sr25519::Signature) -> dispatch::DispatchResult {

            let who = ensure_signed(origin)?;

            let doc_key_hash = T::Hashing::hash_of(&(app_id, &document_id));

            ensure!(!<KPDocumentDataByIdHash<T>>::contains_key(&doc_key_hash), Error::<T>::DocumentAlreadyExisted);

            let model_key = T::Hashing::hash_of(&(app_id, &model_id));
            ensure!(<KPModelDataByIdHash<T>>::contains_key(&model_key), Error::<T>::ModelNotFound);

            // create doc
            let doc = KPDocumentData {
                sender: who.clone(),
                owner: app_user_account.clone(),
                document_type: DocumentType::ModelCreate,
                app_id,
                document_id: document_id.clone(),
                model_id,
                product_id,
                content_hash,
                document_data: DocumentSpecificData::ModelCreate(document_power_data),
                ..Default::default()
            };

            // process content power
            Self::process_document_content_power(&doc);
            Self::process_commodity_power(&doc);

            // create document record
            <KPDocumentDataByIdHash<T>>::insert(&doc_key_hash, &doc);

            Self::deposit_event(RawEvent::KnowledgeCreated(who));
            Ok(())
        }

        #[weight = 0]
        pub fn democracy_slash_commodity_power(origin,
            app_id: u32,
            cart_id: Vec<u8>,
            comment_id: Vec<u8>,
            reporter_account: T::AccountId
            ) -> dispatch::DispatchResult {

            ensure_root(origin)?;

            // read out comment to get related document owner
            let comment_key = T::Hashing::hash_of(&(app_id, &comment_id));
            ensure!(<KPCommentDataByIdHash<T>>::contains_key(&comment_key), Error::<T>::CommentNotFound);
            let comment = <KPCommentDataByIdHash<T>>::get(&comment_key);

            let doc_key = T::Hashing::hash_of(&(app_id, &comment.document_id));
            ensure!(!<KPDocumentDataByIdHash<T>>::contains_key(&doc_key), Error::<T>::DocumentNotFound);
            let doc = <KPDocumentDataByIdHash<T>>::get(&doc_key);

            // get model id from publish doc
            let model_id = Self::get_model_id_from_product(app_id, &doc.product_id).unwrap_or_default();

            // perform slash
            let key_hash = T::Hashing::hash_of(&(app_id, &cart_id));
            let owner_account = Self::convert_account(&doc.owner);
            Self::slash_power(&key_hash, &owner_account);
            Self::remove_leader_board_item(app_id, &model_id, &cart_id);
            // TODO: send benefit to reporter_account

            Self::deposit_event(RawEvent::PowerSlashed(owner_account));
            Ok(())
        }

        #[weight = 0]
        pub fn democracy_model_dispute(origin,
            app_id: u32,
            model_id: Vec<u8>,
            dispute_type: ModelDisputeType,
            comment_id: Vec<u8>,
            reporter_account: T::AccountId
            ) -> dispatch::DispatchResult {

            ensure_root(origin)?;

            // get model creator account
            let key = T::Hashing::hash_of(&(app_id, &model_id));
            ensure!(<KPModelDataByIdHash<T>>::contains_key(&key), Error::<T>::ModelNotFound);

            let model = <KPModelDataByIdHash<T>>::get(&key);
            // according dispute type to decide slash
            let owner_account = Self::convert_account(&model.owner);

            // TODO: according dispute type decide punishment
            match dispute_type {
                ModelDisputeType::NoneIntendNormal => {}
                ModelDisputeType::IntendNormal => {}
                ModelDisputeType::Serious => {}
            }


            // TODO: send benefit to reporter_account

            Self::deposit_event(RawEvent::ModelDisputed(owner_account));
            Ok(())
        }

        #[weight = 0]
        pub fn democracy_add_app(origin, app_type: Vec<u8>, app_name: Vec<u8>,
            app_key: T::AccountId, app_admin_key: T::AccountId, return_rate: u32) -> dispatch::DispatchResult {
            ensure_root(origin)?;
            // check app_type
            ensure!(<AppIdRange>::contains_key(&app_type), Error::<T>::AppTypeInvalid);
            // check return_rate
            ensure!(return_rate > 0 && return_rate < 10000, Error::<T>::ReturnRateInvalid);
            // generate app_id
            let app_id = <AppIdRange>::get(&app_type) + 1;
            // set admin and idenetity key
            T::Membership::config_app_admin(&app_admin_key, app_id);
            T::Membership::config_app_key(&app_key, app_id);
            T::Membership::config_app_setting(app_id, return_rate, app_name);

            // update app_id range store
            <AppIdRange>::insert(&app_type, app_id);

            Self::deposit_event(RawEvent::AppAdded(app_id));
            Ok(())
        }

        #[weight = 0]
        pub fn democracy_app_financed(origin, app_id: u32, kpt_amount: BalanceOf<T>, exchange_rate: BalanceOf<T>) -> dispatch::DispatchResult {
            ensure_root(origin)?;

            ensure!(T::Membership::is_valid_app(app_id), Error::<T>::AppIdInvalid);

            // TODO: exchange_rate min max configuration
            ensure!(exchange_rate >= T::KptExchangeMinRate::get(), Error::<T>::AppFinancedExchangeRateTooLow);

            // Do we permit multi-financed records?
            ensure!(!<AppFinancedRecord<T>>::contains_key(app_id), Error::<T>::AppAlreadyFinanced);

            <AppFinancedRecord<T>>::insert(app_id, AppFinancedData::<T> {
                amount: kpt_amount,
                exchange_rate,
            });

            // TODO: start exchange process

            Self::deposit_event(RawEvent::AppFinanced(app_id));
            Ok(())
        }

        #[weight = 0]
        pub fn create_power_leader_board(origin, app_id: u32, model_id: Vec<u8>) -> dispatch::DispatchResult {
            ensure_root(origin)?;

            let current_block = <system::Module<T>>::block_number();
            // read out last time block number and check distance
            let last_key = T::Hashing::hash_of(&(app_id, model_id));
            let last_block = <AppLeaderBoardLastTime<T>>::get(&last_key);
            let diff = current_block - last_block;
            ensure!(diff < T::AppLeaderBoardInterval::get(), Error::<T>::LeaderBoardCreateNotPermit);


            Self::deposit_event(RawEvent::LeaderBoardsCreated(current_block));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn kp_total_power() -> PowerSize {
        TotalPower::get()
    }

    pub fn kp_account_power(account: T::AccountId) -> PowerSize {
        <MinerPowerByAccount<T>>::get(account)
    }

    pub fn kp_auth_account_power(account: AuthAccountId) -> PowerSize {
        let account_id = Self::convert_account(&account);
        Self::kp_account_power(account_id)
    }

    pub fn kp_account_power_ratio(account: &T::AccountId) -> f64 {
        let account_power = <MinerPowerByAccount<T>>::get(account) as f64;
        let total_power = Self::kp_total_power();
        if total_power == 0 {
            0.0
        } else {
            account_power / total_power as f64
        }
    }

    pub fn kp_commodity_power(app_id: u32, cart_id: Vec<u8>) -> PowerSize {
        let key = T::Hashing::hash_of(&(app_id, &cart_id));
        Self::get_purchase_power(&key)
    }

    pub fn kp_is_commodity_power_exist(app_id: u32, cart_id: Vec<u8>) -> bool {
        let key = T::Hashing::hash_of(&(app_id, &cart_id));
        <KPPurchasePowerByIdHash<T>>::contains_key(&key)
    }

    pub fn kp_document_power(app_id: u32, document_id: Vec<u8>) -> DocumentPower {
        let key = T::Hashing::hash_of(&(app_id, &document_id));
        <KPDocumentPowerByIdHash<T>>::get(&key)
    }

    pub fn kp_account_attend_power(app_id: u32, account: T::AccountId) -> PowerSize {
        let key = T::Hashing::hash_of(&(&account, app_id));
        <AccountAttendPowerMap<T>>::get(&key)
    }

    pub fn kp_get_commodity_types() -> Vec<CommodityTypeData> {
        CommodityTypeSets::get()
    }

    fn is_auth_server(who: &T::AccountId) -> bool {
        <AuthServers<T>>::get().contains(who)
    }

    fn auth_server_verify(server: AuthAccountId, sign: sr25519::Signature, msg: &[u8]) -> bool {
        let ms: MultiSignature = sign.into();
        ms.verify(msg, &server)
    }

    fn convert_account(origin: &AuthAccountId) -> T::AccountId {
        let tmp: [u8; 32] = origin.clone().into();
        T::AccountId::decode(&mut &tmp[..]).unwrap_or_default()
    }

    fn compute_commodity_power(power: &CommodityPowerSet) -> PowerSize {
        power.0.total() + power.1.total() + power.2.total() + power.3 + power.4
    }

    fn get_purchase_power(key: &T::Hash) -> PowerSize {
        let power = <KPPurchasePowerByIdHash<T>>::get(key);
        Self::compute_commodity_power(&power)
    }

    fn update_purchase_power(
        key: &T::Hash,
        power_set: &CommodityPowerSet,
        app_id: u32,
        model_id: &Vec<u8>,
        cart_id: &Vec<u8>,
    ) {
        let is_slashed = <KPPurchaseBlackList<T>>::contains_key(key);
        if !is_slashed {
            <KPPurchasePowerByIdHash<T>>::insert(&key, power_set);
            let power = Self::compute_commodity_power(power_set);
            // update model board
            Self::update_realtime_power_leader_boards(app_id, model_id, cart_id, power);
            // uupdate app board
            Self::update_realtime_power_leader_boards(app_id, &vec![], cart_id, power);
        }
    }

    fn clear_purchase_power(key: &T::Hash) {
        let empty_power = DocumentPower {
            attend: 0,
            content: 0,
            judge: 0,
        };
        <KPPurchasePowerByIdHash<T>>::insert(
            &key,
            &(
                empty_power.clone(),
                empty_power.clone(),
                empty_power.clone(),
                0,
                0,
            ),
        );
        <KPPurchaseBlackList<T>>::insert(key, true);
    }

    fn get_leader_item(
        group_key: &T::Hash,
        cart_id: &Vec<u8>,
    ) -> Option<(usize, CommodityLeaderBoardData<T>)> {
        if !<LeaderBoardCommoditySet<T>>::contains_key(group_key, cart_id) {
            return None;
        } else {
            // go through specified leader board
            let hash = T::Hashing::hash_of(cart_id);
            let board = <AppModelCommodityLeaderBoards<T>>::get(group_key);
            for (pos, item) in board.iter().enumerate() {
                if item.cart_id_hash == hash {
                    return Some((pos, item.clone()));
                }
            }
        }

        None
    }

    fn get_model_id_from_product(app_id: u32, product_id: &Vec<u8>) -> Option<Vec<u8>> {
        // get model id from publish doc
        let publish_key = T::Hashing::hash_of(&(app_id, &product_id));
        if !<KPDocumentProductIndexByIdHash<T>>::contains_key(&publish_key) {
            return None;
        }
        let publish_doc_id = <KPDocumentProductIndexByIdHash<T>>::get(&publish_key);
        let publish_doc_key = T::Hashing::hash_of(&(app_id, &publish_doc_id));
        if !<KPDocumentDataByIdHash<T>>::contains_key(&publish_doc_key) {
            return None;
        }
        let publish_doc = <KPDocumentDataByIdHash<T>>::get(&publish_doc_key);

        Some(publish_doc.model_id)
    }

    fn remove_leader_board_item(app_id: u32, model_id: &Vec<u8>, cart_id: &Vec<u8>) -> Option<u32> {
        let key = T::Hashing::hash_of(&(app_id, model_id));

        let (index, _) = Self::get_leader_item(&key, cart_id)?;

        let mut board = <AppModelCommodityLeaderBoards<T>>::get(&key);
        board.remove(index);
        <AppModelCommodityLeaderBoards<T>>::insert(&key, board);

        Some(0)
    }

    fn compute_publish_product_content_power(
        para_issue_rate: f64,
        self_issue_rate: f64,
    ) -> PowerSize {
        ((para_issue_rate * T::DocumentPublishWeightParamsRate::get() as f64 / RATIO_DIV
            + self_issue_rate * T::DocumentPublishWeightParamsSelfRate::get() as f64 / RATIO_DIV)
            * T::DocumentPowerWeightContent::get() as f64
            / RATIO_DIV
            * T::TopWeightProductPublish::get() as f64
            / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize
    }

    fn compute_identify_content_power(ident_rate: f64, ident_consistence: f64) -> PowerSize {
        ((ident_rate * T::DocumentIdentifyWeightParamsRate::get() as f64 / RATIO_DIV
            + ident_consistence * T::DocumentIdentifyWeightCheckRate::get() as f64 / RATIO_DIV)
            * T::DocumentPowerWeightContent::get() as f64
            / RATIO_DIV
            * T::TopWeightDocumentIdentify::get() as f64
            / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize
    }

    fn compute_try_content_power(offset_rate: f64, true_rate: f64) -> PowerSize {
        ((offset_rate * T::DocumentTryWeightBiasRate::get() as f64 / RATIO_DIV
            + true_rate * T::DocumentTryWeightTrueRate::get() as f64 / RATIO_DIV)
            * T::DocumentPowerWeightContent::get() as f64
            / RATIO_DIV
            * T::TopWeightDocumentTry::get() as f64
            / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize
    }

    fn compute_choose_content_power(sell_count_rate: f64, try_count_rate: f64) -> PowerSize {
        ((sell_count_rate * T::DocumentChooseWeightSellCount::get() as f64 / RATIO_DIV
            + try_count_rate * T::DocumentChooseWeightTryCount::get() as f64 / RATIO_DIV)
            * T::DocumentCMPowerWeightContent::get() as f64
            / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize
    }

    fn compute_model_content_power(producer_count_rate: f64, product_count_rate: f64) -> PowerSize {
        ((producer_count_rate * T::DocumentModelWeightProducerCount::get() as f64 / RATIO_DIV
            + product_count_rate * T::DocumentModelWeightProductCount::get() as f64 / RATIO_DIV)
            * T::DocumentCMPowerWeightContent::get() as f64
            / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize
    }

    fn compute_attend_power(
        rates: (f64, f64, f64, f64),
        second_weight: PowerSize,
        top_weight: PowerSize,
    ) -> PowerSize {
        ((rates.0 * T::CommentPowerWeightCount::get() as f64 / RATIO_DIV
            + rates.1 * T::CommentPowerWeightCost::get() as f64 / RATIO_DIV
            + rates.2 * T::CommentPowerWeightPerCost::get() as f64 / RATIO_DIV
            + rates.3 * T::CommentPowerWeightPositive::get() as f64 / RATIO_DIV)
            * second_weight as f64
            / RATIO_DIV
            * top_weight as f64
            / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize
    }

    fn compute_judge_power(
        origin_power: f64,
        top_weight: PowerSize,
        document_weight: u8,
    ) -> PowerSize {
        (origin_power * document_weight as f64 / RATIO_DIV * top_weight as f64 / RATIO_DIV
            * FLOAT_COMPUTE_PRECISION as f64) as PowerSize
    }

    fn compute_comment_action_rate(
        max: &CommentMaxRecord,
        count: PowerSize,
        fee: PowerSize,
        positive: PowerSize,
        unit_fee: PowerSize,
    ) -> (f64, f64, f64, f64) {
        let mut positive_rate: f64 = 0.0;
        let count_rate = count as f64 / max.max_count as f64;
        let cost_rate = fee as f64 / max.max_fee as f64;
        let unit_cost_rate = unit_fee as f64 / max.max_unit_fee as f64;

        if max.max_positive > 0 {
            positive_rate = positive as f64 / max.max_positive as f64;
        }

        (count_rate, cost_rate, unit_cost_rate, positive_rate)
    }

    fn update_max<F>(rate: PowerSize, mut max: PowerSize, updater: F) -> f64
    where
        F: Fn(PowerSize) -> (),
    {
        if rate > max {
            max = rate;
            updater(max);
        }

        if rate > 0 {
            return rate as f64 / max as f64;
        }

        0.0
    }

    fn update_comment_max(
        max: &mut CommentMaxRecord,
        count: PowerSize,
        fees: PowerSize,
        positive: PowerSize,
        unit_fee: PowerSize,
    ) -> bool {
        let mut is_updated = false;

        if count > max.max_count {
            max.max_count = count;
            is_updated = true;
        }
        if fees > max.max_fee {
            max.max_fee = fees;
            is_updated = true;
        }
        if positive > max.max_positive {
            max.max_positive = positive;
            is_updated = true;
        }
        if unit_fee > max.max_unit_fee {
            max.max_unit_fee = unit_fee;
            is_updated = true;
        }

        is_updated
    }

    fn compute_doc_trend_power(doc: &KPDocumentData<T::AccountId, T::Hash>) -> f64 {
        match doc {
            KPDocumentData {
                expert_trend,
                platform_trend,
                ..
            } => {
                let et = *expert_trend as u8;
                let pt = *platform_trend as u8;

                match et ^ pt {
                    // 01 10, 10 01  single negative
                    0b11 => 0.25,
                    // 00 00, 01 01, 10 10
                    0b00 => match et & pt {
                        0b00 => 1.0,
                        0b01 => 0.0,
                        0b10 => 0.375,
                        // unexpected!!!
                        _ => {
                            print("unexpected");
                            0.0
                        }
                    },
                    // 00 01, 01 00 positive and negative
                    0b01 => 0.5,
                    // 00 10, 10 00 single positive
                    0b10 => 0.75,
                    // unexpected!!!
                    _ => {
                        print("unexpected");
                        0.0
                    }
                }
            }
        }
    }

    fn update_realtime_power_leader_boards(
        app_id: u32,
        model_id: &Vec<u8>,
        cart_id: &Vec<u8>,
        power: PowerSize,
    ) -> Option<u32> {
        // get leader board
        let leader_key = T::Hashing::hash_of(&(app_id, model_id));
        let mut board = <AppModelCommodityLeaderBoards<T>>::get(&leader_key);

        let board_item = CommodityLeaderBoardData {
            cart_id_hash: T::Hashing::hash_of(cart_id),
            cart_id: cart_id.clone(),
            power,
        };

        // check leader set double map to make sure this item is already in leader board
        match Self::get_leader_item(&leader_key, cart_id) {
            Some((index, org_item)) => {
                if org_item == board_item {
                    // power no change, do nothing
                } else {
                    // remove old one, reinsert
                    board.remove(index);
                }
            }
            None => {}
        }

        // now we can find the proper position for this new power item
        match board.binary_search_by(|probe| probe.cmp(&board_item).reverse()) {
            Ok(index) => {
                // always get the end
                board.insert(index + 1, board_item);
            }
            Err(index) => {
                // not found, index is closet position(upper)
                board.insert(index, board_item);
            }
        }

        // update leader set
        <LeaderBoardCommoditySet<T>>::insert(&leader_key, cart_id, ());

        // check if reach board max
        if T::AppLeaderBoardMaxPos::get() < board.len() as u32 {
            print("leader board full, drop last one");
            let removed = board.pop()?;
            <LeaderBoardCommoditySet<T>>::remove(&leader_key, removed.cart_id);
        }

        Some(0)
    }

    fn increase_commodity_count(
        app_id: u32,
        model_id: &Vec<u8>,
        cart_id: &Vec<u8>,
        doc_type: DocumentType,
    ) {
        match doc_type {
            DocumentType::ProductTry => {
                // check if another identify document exist
                let key = T::Hashing::hash_of(&(app_id, cart_id));
                if <KPCartProductIdentifyIndexByIdHash<T>>::contains_key(&key) {
                    return;
                }
                <AppCommodityCount>::mutate(app_id, |count| {
                    *count = *count + 1;
                });
                let model_key = T::Hashing::hash_of(&(app_id, model_id));
                <AppModelCommodityCount<T>>::mutate(&model_key, |count| {
                    *count = *count + 1;
                });
            }
            DocumentType::ProductIdentify => {
                let key = T::Hashing::hash_of(&(app_id, cart_id));
                if <KPCartProductTryIndexByIdHash<T>>::contains_key(&key) {
                    return;
                }
                <AppCommodityCount>::mutate(app_id, |count| {
                    *count = *count + 1;
                });
                let model_key = T::Hashing::hash_of(&(app_id, model_id));
                <AppModelCommodityCount<T>>::mutate(&model_key, |count| {
                    *count = *count + 1;
                });
            }
            _ => {}
        }
    }

    fn leader_board_lottery(block: T::BlockNumber, app_id: u32, model_id: &Vec<u8>) {
        let seed = T::Randomness::random(b"ctt_power");
        // seed needs to be guaranteed to be 32 bytes.
        let seed = <[u8; 32]>::decode(&mut TrailingZeroInput::new(seed.as_ref()))
            .expect("input is padded with zeroes; qed");
        let mut rng = ChaChaRng::from_seed(seed);

        //pick_item(&mut rng, &votes)
    }

    fn update_document_comment_pool(
        new_comment: &KPCommentData<T::AccountId, T::Hash>,
        doc: &KPDocumentData<T::AccountId, T::Hash>,
    ) {
        let key = T::Hashing::hash_of(&(doc.app_id, &doc.document_id));
        let mut pool = <DocumentCommentsAccountPool<T>>::get(&key);

        let pool_item = CommentWeightData {
            account: new_comment.sender.clone(),
            position: doc.comment_count,
            cash_cost: new_comment.comment_fee,
        };

        /*match pool.binary_search(&pool_item) {
            Ok(index) => {
                // remove old and push to end
                pool.remove(index);
            }
            Err(_index) => {}
        }*/

        pool.push(pool_item);
        <DocumentCommentsAccountPool<T>>::insert(&key, pool);
    }

    fn insert_document_power(
        doc: &KPDocumentData<T::AccountId, T::Hash>,
        content_power: PowerSize,
        judge_power: PowerSize,
    ) {
        let key = T::Hashing::hash_of(&(doc.app_id, &doc.document_id));
        let power = DocumentPower {
            attend: 0,
            content: content_power,
            judge: judge_power,
        };

        <KPDocumentPowerByIdHash<T>>::insert(&key, &power);
        Self::add_accumulation_document_power(&power, doc);
    }

    fn update_document_power(
        doc: &KPDocumentData<T::AccountId, T::Hash>,
        attend_power: PowerSize,
        judge_power: PowerSize,
    ) -> Option<u32> {
        // read out original first
        let key = T::Hashing::hash_of(&(doc.app_id, &doc.document_id));
        let mut org_power = <KPDocumentPowerByIdHash<T>>::get(&key);
        let mut is_need_accumulation = false;
        let commodity_key;
        let mut commodity_power;

        match &doc.document_data {
            DocumentSpecificData::ProductIdentify(data) => {
                is_need_accumulation = true;

                commodity_key = T::Hashing::hash_of(&(doc.app_id, &data.cart_id));
                let is_slashed = <KPPurchaseBlackList<T>>::contains_key(&commodity_key);
                if !is_slashed {
                    commodity_power = <KPPurchasePowerByIdHash<T>>::get(&commodity_key);
                    commodity_power.1 = DocumentPower {
                        attend: attend_power,
                        content: commodity_power.1.content,
                        judge: judge_power,
                    };

                    let model_id = Self::get_model_id_from_product(doc.app_id, &doc.product_id)?;

                    Self::update_purchase_power(
                        &commodity_key,
                        &commodity_power,
                        doc.app_id,
                        &model_id,
                        &data.cart_id,
                    );
                }
            }
            DocumentSpecificData::ProductTry(data) => {
                is_need_accumulation = true;
                commodity_key = T::Hashing::hash_of(&(doc.app_id, &data.cart_id));
                let is_slashed = <KPPurchaseBlackList<T>>::contains_key(&commodity_key);
                if !is_slashed {
                    commodity_power = <KPPurchasePowerByIdHash<T>>::get(&commodity_key);
                    commodity_power.2 = DocumentPower {
                        attend: attend_power,
                        content: commodity_power.2.content,
                        judge: judge_power,
                    };

                    let model_id = Self::get_model_id_from_product(doc.app_id, &doc.product_id)?;
                    Self::update_purchase_power(
                        &commodity_key,
                        &commodity_power,
                        doc.app_id,
                        &model_id,
                        &data.cart_id,
                    );
                }
            }
            _ => {}
        }

        if is_need_accumulation {
            let accumulation_key = T::Hashing::hash_of(&(doc.app_id, &doc.owner));
            let mut accumulation_power =
                <MinerDocumentsAccumulationPower<T>>::get(&accumulation_key);
            // params only > 0 means valid
            if attend_power > 0 {
                accumulation_power.attend = min(0, accumulation_power.attend - org_power.attend);
                accumulation_power.attend += attend_power;
            }

            if judge_power > 0 {
                accumulation_power.judge = min(0, accumulation_power.judge - org_power.judge);
                accumulation_power.judge += judge_power;
            }
            <MinerDocumentsAccumulationPower<T>>::insert(&accumulation_key, accumulation_power);
        }

        if attend_power > 0 {
            org_power.attend = attend_power;
        }

        if judge_power > 0 {
            org_power.judge = judge_power;
        }

        // update store
        <KPDocumentPowerByIdHash<T>>::insert(&key, org_power);

        Some(0)
    }

    // only for identify and try doc
    fn add_accumulation_document_power(
        new_power: &DocumentPower,
        doc: &KPDocumentData<T::AccountId, T::Hash>,
    ) -> Option<u32> {
        // check if cart_id matched another doc exist
        let is_need_add_publish: bool;
        let commodity_key;
        let mut commodity_power;
        let cart_id: Vec<u8>;

        match &doc.document_data {
            DocumentSpecificData::ProductIdentify(data) => {
                // check if exist product try
                commodity_key = T::Hashing::hash_of(&(doc.app_id, &data.cart_id));
                is_need_add_publish =
                    !<KPCartProductTryIndexByIdHash<T>>::contains_key(&commodity_key);

                commodity_power = <KPPurchasePowerByIdHash<T>>::get(&commodity_key);
                commodity_power.1 = new_power.clone();
                cart_id = data.cart_id.clone();
            }
            DocumentSpecificData::ProductTry(data) => {
                commodity_key = T::Hashing::hash_of(&(doc.app_id, &data.cart_id));
                is_need_add_publish =
                    !<KPCartProductIdentifyIndexByIdHash<T>>::contains_key(&commodity_key);
                commodity_power = <KPPurchasePowerByIdHash<T>>::get(&commodity_key);
                commodity_power.2 = new_power.clone();
                cart_id = data.cart_id.clone();
            }
            _ => {
                return None;
            }
        }

        let mut publish_power = DocumentPower {
            attend: 0,
            content: 0,
            judge: 0,
        };

        if is_need_add_publish {
            // add doc matched publish power
            let publish_doc_key = T::Hashing::hash_of(&(doc.app_id, &doc.product_id));
            let publish_doc_id = <KPDocumentProductIndexByIdHash<T>>::get(&publish_doc_key);
            // read out publish document power
            let publish_power_key = T::Hashing::hash_of(&(doc.app_id, &publish_doc_id));
            publish_power = <KPDocumentPowerByIdHash<T>>::get(&publish_power_key);
            commodity_power.0 = publish_power.clone();
        }

        let is_slashed = <KPPurchaseBlackList<T>>::contains_key(&commodity_key);
        if !is_slashed {
            let model_id = Self::get_model_id_from_product(doc.app_id, &doc.product_id)?;
            Self::update_purchase_power(
                &commodity_key,
                &commodity_power,
                doc.app_id,
                &model_id,
                &cart_id,
            );
        }

        let key = T::Hashing::hash_of(&(doc.app_id, &doc.owner));
        let current = <MinerDocumentsAccumulationPower<T>>::get(&key);
        <MinerDocumentsAccumulationPower<T>>::insert(&key, &current + &new_power + publish_power);

        Some(0)
    }

    fn process_document_content_power(doc: &KPDocumentData<T::AccountId, T::Hash>) {
        let content_power;
        let initial_judge_power;
        match &doc.document_data {
            DocumentSpecificData::ProductPublish(data) => {
                let params_max = <DocumentPublishMaxParams>::get(doc.app_id);
                let para_issue_rate_p =
                    Self::update_max(data.para_issue_rate, params_max.para_issue_rate, |v| {
                        <DocumentPublishMaxParams>::mutate(doc.app_id, |max| {
                            max.para_issue_rate = v;
                        })
                    });

                let self_issue_rate_p =
                    Self::update_max(data.self_issue_rate, params_max.self_issue_rate, |v| {
                        <DocumentPublishMaxParams>::mutate(doc.app_id, |max| {
                            max.self_issue_rate = v;
                        })
                    });

                // compute power
                content_power = Self::compute_publish_product_content_power(
                    para_issue_rate_p,
                    self_issue_rate_p,
                );

                initial_judge_power = Self::compute_judge_power(
                    Self::compute_doc_trend_power(&doc),
                    T::TopWeightProductPublish::get() as PowerSize,
                    T::DocumentPowerWeightJudge::get(),
                );
                Self::insert_document_power(&doc, content_power, initial_judge_power);
            }
            DocumentSpecificData::ProductIdentify(data) => {
                let params_max = <DocumentIdentifyMaxParams>::get(doc.app_id);
                let ident_rate_p = Self::update_max(data.ident_rate, params_max.ident_rate, |v| {
                    <DocumentIdentifyMaxParams>::mutate(doc.app_id, |max| {
                        max.ident_rate = v;
                    })
                });

                let ident_consistence_p =
                    Self::update_max(data.ident_consistence, params_max.ident_consistence, |v| {
                        <DocumentIdentifyMaxParams>::mutate(doc.app_id, |max| {
                            max.ident_consistence = v;
                        })
                    });

                content_power =
                    Self::compute_identify_content_power(ident_rate_p, ident_consistence_p);

                initial_judge_power = Self::compute_judge_power(
                    Self::compute_doc_trend_power(&doc),
                    T::TopWeightDocumentIdentify::get() as PowerSize,
                    T::DocumentPowerWeightJudge::get(),
                );
                Self::insert_document_power(&doc, content_power, initial_judge_power);
            }
            DocumentSpecificData::ProductTry(data) => {
                let params_max = <DocumentTryMaxParams>::get(doc.app_id);
                let offset_rate_p =
                    Self::update_max(data.offset_rate, params_max.offset_rate, |v| {
                        <DocumentTryMaxParams>::mutate(doc.app_id, |max| {
                            max.offset_rate = v;
                        })
                    });

                let true_rate_p = Self::update_max(data.true_rate, params_max.true_rate, |v| {
                    <DocumentTryMaxParams>::mutate(doc.app_id, |max| {
                        max.true_rate = v;
                    })
                });

                content_power = Self::compute_try_content_power(offset_rate_p, true_rate_p);

                initial_judge_power = Self::compute_judge_power(
                    Self::compute_doc_trend_power(&doc),
                    T::TopWeightDocumentTry::get() as PowerSize,
                    T::DocumentPowerWeightJudge::get(),
                );
                Self::insert_document_power(&doc, content_power, initial_judge_power);
            }
            DocumentSpecificData::ProductChoose(data) => {
                let params_max = <DocumentChooseMaxParams>::get(doc.app_id);
                let sell_count_p = Self::update_max(data.sell_count, params_max.sell_count, |v| {
                    <DocumentChooseMaxParams>::mutate(doc.app_id, |max| {
                        max.sell_count = v;
                    })
                });

                let try_count_p = Self::update_max(data.try_count, params_max.try_count, |v| {
                    <DocumentChooseMaxParams>::mutate(doc.app_id, |max| {
                        max.try_count = v;
                    })
                });

                content_power = Self::compute_choose_content_power(sell_count_p, try_count_p);

                initial_judge_power = Self::compute_judge_power(
                    Self::compute_doc_trend_power(&doc),
                    100 as PowerSize,
                    T::DocumentCMPowerWeightJudge::get(),
                );
                Self::insert_document_power(&doc, content_power, initial_judge_power);
            }
            DocumentSpecificData::ModelCreate(data) => {
                let params_max = <DocumentModelCreateMaxParams>::get(doc.app_id);
                let producer_count_p =
                    Self::update_max(data.producer_count, params_max.producer_count, |v| {
                        <DocumentModelCreateMaxParams>::mutate(doc.app_id, |max| {
                            max.producer_count = v;
                        })
                    });

                let product_count_p =
                    Self::update_max(data.product_count, params_max.product_count, |v| {
                        <DocumentModelCreateMaxParams>::mutate(doc.app_id, |max| {
                            max.product_count = v;
                        })
                    });

                content_power =
                    Self::compute_model_content_power(producer_count_p, product_count_p);

                initial_judge_power = Self::compute_judge_power(
                    Self::compute_doc_trend_power(&doc),
                    100 as PowerSize,
                    T::DocumentCMPowerWeightJudge::get(),
                );
                Self::insert_document_power(&doc, content_power, initial_judge_power);
            }
        }
    }

    fn process_comment_power(comment: &KPCommentData<T::AccountId, T::Hash>) {
        // target compute
        let account_comment_power: PowerSize;
        let doc_comment_power: PowerSize;
        let doc_key_hash = T::Hashing::hash_of(&(comment.app_id, &comment.document_id));

        // read out document
        let mut doc = Self::kp_document_data_by_idhash(&doc_key_hash);

        let comment_account_key = T::Hashing::hash_of(&(comment.app_id, &comment.sender));
        let mut account = Self::kp_comment_account_record_map(&comment_account_key);

        account.count += 1;
        account.fees += comment.comment_fee;

        doc.comment_count += 1;
        doc.comment_total_fee += comment.comment_fee;
        if comment.comment_trend == 0 {
            doc.comment_positive_count += 1;
            account.positive_count += 1;
        }

        let mut account_comment_max = Self::comment_max_info_per_account_map(comment.app_id);

        let account_comment_unit_fee = account.fees / account.count;
        let is_account_max_updated = Self::update_comment_max(
            &mut account_comment_max,
            account.count,
            account.fees,
            account.positive_count,
            account_comment_unit_fee,
        );

        let mut account_attend_weight: PowerSize = 0;
        let mut comment_power_weight: PowerSize = 0;
        let mut doc_comment_top_weight: PowerSize = 0;
        // according doc type to decide weight
        match doc.document_type {
            DocumentType::ProductPublish => {
                account_attend_weight = T::TopWeightAccountAttend::get() as PowerSize;
                comment_power_weight = T::DocumentPowerWeightAttend::get() as PowerSize;
                doc_comment_top_weight = T::TopWeightProductPublish::get() as PowerSize;
            }
            DocumentType::ProductIdentify => {
                account_attend_weight = T::TopWeightAccountAttend::get() as PowerSize;
                comment_power_weight = T::DocumentPowerWeightAttend::get() as PowerSize;
                doc_comment_top_weight = T::TopWeightDocumentIdentify::get() as PowerSize;
            }
            DocumentType::ProductTry => {
                account_attend_weight = T::TopWeightAccountAttend::get() as PowerSize;
                comment_power_weight = T::DocumentPowerWeightAttend::get() as PowerSize;
                doc_comment_top_weight = T::TopWeightDocumentTry::get() as PowerSize;
            }
            DocumentType::ProductChoose => {
                account_attend_weight = T::CMPowerAccountAttend::get() as PowerSize;
                comment_power_weight = T::DocumentCMPowerWeightAttend::get() as PowerSize;
                doc_comment_top_weight = 100 as PowerSize;
            }
            DocumentType::ModelCreate => {
                account_attend_weight = T::CMPowerAccountAttend::get() as PowerSize;
                comment_power_weight = T::DocumentCMPowerWeightAttend::get() as PowerSize;
                doc_comment_top_weight = 100 as PowerSize;
            }
            _ => {}
        }

        account_comment_power = Self::compute_attend_power(
            Self::compute_comment_action_rate(
                &account_comment_max,
                account.count,
                account.fees,
                account.positive_count,
                account_comment_unit_fee,
            ),
            100,
            account_attend_weight,
        );

        // read out document based max record
        let mut doc_comment_max = Self::comment_max_info_per_doc_map(comment.app_id);
        let doc_comment_unit_fee = doc.comment_total_fee / doc.comment_count;
        let is_doc_max_updated = Self::update_comment_max(
            &mut doc_comment_max,
            doc.comment_count,
            doc.comment_total_fee,
            doc.comment_positive_count,
            doc_comment_unit_fee,
        );

        // compute document attend power
        doc_comment_power = Self::compute_attend_power(
            Self::compute_comment_action_rate(
                &doc_comment_max,
                doc.comment_count,
                doc.comment_total_fee,
                doc.comment_positive_count,
                doc_comment_unit_fee,
            ),
            comment_power_weight,
            doc_comment_top_weight,
        );

        // chcek if owner's membership
        let mut platform_comment_power: PowerSize = 0;
        if doc.expert_trend == CommentTrend::Empty
            && T::Membership::is_expert(&comment.sender, doc.app_id, &doc.model_id)
        {
            doc.expert_trend = comment.comment_trend.into();
            platform_comment_power =
                (Self::compute_doc_trend_power(&doc) * FLOAT_COMPUTE_PRECISION as f64) as PowerSize;
        }
        if doc.platform_trend == CommentTrend::Empty
            && T::Membership::is_platform(&comment.sender, doc.app_id)
        {
            doc.platform_trend = comment.comment_trend.into();
            platform_comment_power =
                (Self::compute_doc_trend_power(&doc) * FLOAT_COMPUTE_PRECISION as f64) as PowerSize;
        }

        // below are write actions

        // update document record

        <KPDocumentDataByIdHash<T>>::insert(&doc_key_hash, &doc);

        // update account record
        <KPCommentAccountRecordMap<T>>::insert(&comment_account_key, &account);

        // update account max if changed
        if is_account_max_updated {
            <CommentMaxInfoPerAccountMap>::insert(comment.app_id, account_comment_max);
        }

        // update doc comment max if changed
        if is_doc_max_updated {
            <CommentMaxInfoPerDocMap>::insert(comment.app_id, doc_comment_max);
        }

        // update account attend power store
        let key = T::Hashing::hash_of(&(&comment.sender, comment.app_id));
        // is sender a miner? (miner power exist)
        if <MinerPowerByAccount<T>>::contains_key(&comment.sender) {
            let org_miner_power = <MinerPowerByAccount<T>>::get(&comment.sender);
            let org_attend_power = <AccountAttendPowerMap<T>>::get(&key);
            if account_comment_power > org_attend_power {
                <MinerPowerByAccount<T>>::insert(
                    &comment.sender,
                    org_miner_power + account_comment_power - org_attend_power,
                );
            }
        }

        <AccountAttendPowerMap<T>>::insert(&key, account_comment_power);

        // update document attend power store
        Self::update_document_power(&doc, doc_comment_power, platform_comment_power);

        Self::update_document_comment_pool(&comment, &doc);
    }

    // triggered when:
    // 1. doc identify/try/choose/model was created
    // 2. any document was commented, doc param is comment target
    fn process_commodity_power(doc: &KPDocumentData<T::AccountId, T::Hash>) -> Option<u32> {
        let mut power: PowerSize = 0;
        let mut is_need_update_account_power = false;
        let commodity_key;
        let mut commodity_power;
        // read document owner action power
        let key = T::Hashing::hash_of(&(&Self::convert_account(&doc.owner), doc.app_id));
        let owner_account_power = Self::account_attend_power_map(&key);
        power += owner_account_power;

        // TODO: read document owner eocnomic power

        match &doc.document_data {
            DocumentSpecificData::ProductIdentify(data) => {
                is_need_update_account_power = true;
                commodity_key = T::Hashing::hash_of(&(doc.app_id, &data.cart_id));
                let is_slashed = <KPPurchaseBlackList<T>>::contains_key(&commodity_key);
                if !is_slashed {
                    commodity_power = <KPPurchasePowerByIdHash<T>>::get(&commodity_key);
                    // update commodity power
                    commodity_power.3 = owner_account_power;

                    let model_id = Self::get_model_id_from_product(doc.app_id, &doc.product_id)?;

                    Self::update_purchase_power(
                        &commodity_key,
                        &commodity_power,
                        doc.app_id,
                        &model_id,
                        &data.cart_id,
                    );
                }
            }
            DocumentSpecificData::ProductTry(data) => {
                is_need_update_account_power = true;
                commodity_key = T::Hashing::hash_of(&(doc.app_id, &data.cart_id));
                let is_slashed = <KPPurchaseBlackList<T>>::contains_key(&commodity_key);
                if !is_slashed {
                    commodity_power = <KPPurchasePowerByIdHash<T>>::get(&commodity_key);
                    // update commodity power
                    commodity_power.3 = owner_account_power;

                    let model_id = Self::get_model_id_from_product(doc.app_id, &doc.product_id)?;
                    Self::update_purchase_power(
                        &commodity_key,
                        &commodity_power,
                        doc.app_id,
                        &model_id,
                        &data.cart_id,
                    );
                }
            }
            // ignore publish doc
            DocumentSpecificData::ProductPublish(_data) => {
                return None;
            }
            // left is product choose and model create doc, only update commodity power
            _ => {}
        }

        // now we got new computed power, check if need to update
        if is_need_update_account_power {
            let power_key_hash = T::Hashing::hash_of(&(doc.app_id, &doc.owner));
            let doc_power = <MinerDocumentsAccumulationPower<T>>::get(&power_key_hash);
            // need update
            <MinerPowerByAccount<T>>::insert(
                Self::convert_account(&doc.owner),
                power + doc_power.total(),
            );
        } else {
            // for product choose and model create
            let power_key_hash = T::Hashing::hash_of(&(doc.app_id, &doc.document_id));
            let doc_power = <KPDocumentPowerByIdHash<T>>::get(&power_key_hash);
            <KPMiscDocumentPowerByIdHash<T>>::insert(&power_key_hash, power + doc_power.total());
        }

        Some(0)
    }

    fn slash_power(cart_key: &T::Hash, power_owner: &T::AccountId) {
        Self::clear_purchase_power(cart_key);
        let cart_power = Self::get_purchase_power(cart_key);
        if cart_power > 0 {
            // reduce account power
            <MinerPowerByAccount<T>>::mutate(power_owner, |pow| {
                if *pow > cart_power {
                    *pow -= cart_power;
                } else {
                    *pow = 0
                }
            });
        }
    }
}

impl<T: Trait> PowerVote<T::AccountId> for Module<T> {
    fn account_power_ratio(account: &T::AccountId) -> f64 {
        Self::kp_account_power_ratio(account)
    }
}

/// Pick an item at pseudo-random from the slice, given the `rng`. `None` iff the slice is empty.
fn pick_item<'a, R: RngCore, T>(rng: &mut R, items: &'a [T]) -> Option<&'a T> {
    if items.is_empty() {
        None
    } else {
        Some(&items[pick_usize(rng, items.len() - 1)])
    }
}

/// Pick a new PRN, in the range [0, `max`] (inclusive).
fn pick_usize<'a, R: RngCore>(rng: &mut R, max: usize) -> usize {
    (rng.next_u32() % (max as u32 + 1)) as usize
}
