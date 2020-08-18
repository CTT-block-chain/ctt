#![cfg_attr(not(feature = "std"), no_std)]

//! A pallet that implements a storage set on top of a sorted vec and demonstrates performance
//! tradeoffs when using map sets.

use frame_support::{
    codec::Decode,
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, ExistenceRequirement::KeepAlive, Get},
};
use frame_system::{self as system, ensure_root, ensure_signed};
use primitives::{AccountSet, AuthAccountId, Membership};
use sp_core::sr25519;
use sp_runtime::traits::Hash;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

#[cfg(test)]
mod tests;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

pub trait Trait: system::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Currency: Currency<Self::AccountId>;
    type ModelCreatorCreateBenefit: Get<BalanceOf<Self>>;
}

decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
        <T as frame_system::Trait>::AccountId,
    {
        /// Added a member2222
        MemberAdded(AccountId),
        /// Removed a member
        MemberRemoved(AccountId),
        AppAdminSet(AccountId),
        ModleCreatorAdded(AccountId),
        NewUserBenefitDrpped(AccountId, Balance),
    }
);

decl_storage! {
    trait Store for Module<T: Trait> as Members {
        // The set of platform council members. Stored as a single vec, system level
        CouncilMembers get(fn council_members): Vec<T::AccountId>;

        // Finance members, system levele
        FinanceMembers get(fn finance_members): Vec<T::AccountId>;

        // app level admin members key is app_id
        AppAdmins get(fn app_admins):
            map hasher(twox_64_concat) Vec<u8> => T::AccountId;

        // app level platform comment experts, key is app_id, managed by app_admins
        AppPlatformExpertMembers get(fn app_platform_expert_members):
            map hasher(twox_64_concat) Vec<u8> => Vec<T::AccountId>;

        // The set of model creators. Stored as a map, key is app_id & model id
        ModelCreators get(fn model_creators):
            map hasher(twox_64_concat) T::Hash => T::AccountId;

        // key is app_id & model_id, this hash is managed by model_creator
        ExpertMembers get(fn expert_memebers):
            map hasher(twox_64_concat) T::Hash => Vec<T::AccountId>;

        // app_id model_id account -> u32
        ExpertMemberProfitRate get(fn expert_member_profit_rate):
            map hasher(twox_64_concat) T::Hash => u32;

        // app_id user_id -> u32 record first time user KPT drop
        NewAccountBenefitRecords get(fn new_account_benifit_records):
            map hasher(twox_64_concat) T::Hash => BalanceOf<T>;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Cannot join as a member because you are already a member
        AlreadyMember,
        /// Cannot give up membership because you are not currently a member
        NotMember,
        NotAppAdmin,
        NotModelCreator,
        BenefitAlreadyDropped,
        NotEnoughFund,
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        type Error = Error<T>;

        /// Adds a member to the membership set unless the max is reached
        #[weight = 0]
        pub fn add_council_member(origin, new_member: T::AccountId) -> DispatchResult {
            let _who = ensure_root(origin)?;

            let mut members = CouncilMembers::<T>::get();
            //ensure!(members.len() < MAX_MEMBERS, Error::<T>::MembershipLimitReached);

            // We don't want to add duplicate members, so we check whether the potential new
            // member is already present in the list. Because the list is always ordered, we can
            // leverage the binary search which makes this check O(log n).
            match members.binary_search(&new_member) {
                // If the search succeeds, the caller is already a member, so just return
                Ok(_) => Err(Error::<T>::AlreadyMember.into()),
                // If the search fails, the caller is not a member and we learned the index where
                // they should be inserted
                Err(index) => {
                    members.insert(index, new_member.clone());
                    CouncilMembers::<T>::put(members);
                    Self::deposit_event(RawEvent::MemberAdded(new_member));
                    Ok(())
                }
            }
        }

        /// Removes a member.
        #[weight = 0]
        pub fn remove_council_member(origin, old_member: T::AccountId) -> DispatchResult {
            let _who = ensure_root(origin)?;

            let mut members = CouncilMembers::<T>::get();

            // We have to find out if the member exists in the sorted vec, and, if so, where.
            match members.binary_search(&old_member) {
                // If the search succeeds, the caller is a member, so remove her
                Ok(index) => {
                    members.remove(index);
                    CouncilMembers::<T>::put(members);
                    Self::deposit_event(RawEvent::MemberRemoved(old_member));
                    Ok(())
                },
                // If the search fails, the caller is not a member, so just return
                Err(_) => Err(Error::<T>::NotMember.into()),
            }
        }

        #[weight = 0]
        pub fn add_finance_member(origin, new_member: T::AccountId) -> DispatchResult {
            let _who = ensure_root(origin)?;

            let mut members = FinanceMembers::<T>::get();
            //ensure!(members.len() < MAX_MEMBERS, Error::<T>::MembershipLimitReached);

            // We don't want to add duplicate members, so we check whether the potential new
            // member is already present in the list. Because the list is always ordered, we can
            // leverage the binary search which makes this check O(log n).
            match members.binary_search(&new_member) {
                // If the search succeeds, the caller is already a member, so just return
                Ok(_) => Err(Error::<T>::AlreadyMember.into()),
                // If the search fails, the caller is not a member and we learned the index where
                // they should be inserted
                Err(index) => {
                    members.insert(index, new_member.clone());
                    FinanceMembers::<T>::put(members);
                    Self::deposit_event(RawEvent::MemberAdded(new_member));
                    Ok(())
                }
            }
        }

        /// Removes a member.
        #[weight = 0]
        pub fn remove_finance_member(origin, old_member: T::AccountId) -> DispatchResult {
            let _who = ensure_root(origin)?;

            let mut members = FinanceMembers::<T>::get();

            // We have to find out if the member exists in the sorted vec, and, if so, where.
            match members.binary_search(&old_member) {
                // If the search succeeds, the caller is a member, so remove her
                Ok(index) => {
                    members.remove(index);
                    FinanceMembers::<T>::put(members);
                    Self::deposit_event(RawEvent::MemberRemoved(old_member));
                    Ok(())
                },
                // If the search fails, the caller is not a member, so just return
                Err(_) => Err(Error::<T>::NotMember.into()),
            }
        }

        #[weight = 0]
        pub fn set_app_admin(origin, app_id: Vec<u8>, admin: T::AccountId) -> DispatchResult {
            let _who = ensure_root(origin)?;

            <AppAdmins<T>>::insert(app_id, admin.clone());
            Self::deposit_event(RawEvent::AppAdminSet(admin));
            Ok(())
        }

        #[weight = 0]
        pub fn add_app_platform_expert_member(origin, app_id: Vec<u8>, new_member: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // check if origin is app_id's admin
            ensure!(<AppAdmins<T>>::get(&app_id) == who,  Error::<T>::NotAppAdmin);

            let mut members = <AppPlatformExpertMembers<T>>::get(&app_id);

            match members.binary_search(&new_member) {
                // If the search succeeds, the caller is already a member, so just return
                Ok(_) => Err(Error::<T>::AlreadyMember.into()),
                // If the search fails, the caller is not a member and we learned the index where
                // they should be inserted
                Err(index) => {
                    members.insert(index, new_member.clone());
                    <AppPlatformExpertMembers<T>>::insert(&app_id, members);
                    Self::deposit_event(RawEvent::MemberAdded(new_member));
                    Ok(())
                }
            }
        }

        #[weight = 0]
        pub fn remove_app_platform_expert_member(origin, app_id: Vec<u8>, old_member: T::AccountId) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // check if origin is app_id's admin
            ensure!(<AppAdmins<T>>::get(&app_id) == who,  Error::<T>::NotAppAdmin);

            let mut members = <AppPlatformExpertMembers<T>>::get(&app_id);

            match members.binary_search(&old_member) {
                // If the search succeeds, the caller is already a member, so just return
                Ok(index) => {
                    members.remove(index);
                    <AppPlatformExpertMembers<T>>::insert(&app_id, members);
                    Self::deposit_event(RawEvent::MemberRemoved(old_member));
                    Ok(())
                },
                // If the search fails, the caller is not a member, so just return
                Err(_) => Err(Error::<T>::NotMember.into()),
            }
        }

        #[weight = 0]
        pub fn add_expert_member(origin, app_id: Vec<u8>, model_id: Vec<u8>, kpt_profit_rate: u32, model_creator: AuthAccountId, model_creator_sign: sr25519::Signature) -> DispatchResult {
            let new_member = ensure_signed(origin)?;

            // TODO: verify model_sign

            let key = T::Hashing::hash_of(&(&app_id, &model_id));

            let mut members = <ExpertMembers<T>>::get(&key);

            match members.binary_search(&new_member) {
                // If the search succeeds, the caller is already a member, so just return
                Ok(_) => Err(Error::<T>::AlreadyMember.into()),
                // If the search fails, the caller is not a member and we learned the index where
                // they should be inserted
                Err(index) => {
                    members.insert(index, new_member.clone());
                    <ExpertMembers<T>>::insert(&key, members);

                    // update profit rate store
                    let profit_key = T::Hashing::hash_of(&(&app_id, &model_id, &new_member));
                    <ExpertMemberProfitRate<T>>::insert(&profit_key, kpt_profit_rate);

                    Self::deposit_event(RawEvent::MemberAdded(new_member));
                    Ok(())
                }
            }
        }

        #[weight = 0]
        pub fn remove_expert_member(origin,
            old_member: T::AccountId, app_id: Vec<u8>, model_id: Vec<u8>,
            app_user_account: AuthAccountId,
            app_user_sign: sr25519::Signature,

            auth_server: AuthAccountId,
            auth_sign: sr25519::Signature) -> DispatchResult {

            // this is app server account
            let _who = ensure_signed(origin)?;

            // TODO: verify 2 sign

            // check the creator authority
            let key = T::Hashing::hash_of(&(&app_id, &model_id));
            let creator = <ModelCreators<T>>::get(&key);
            ensure!(creator == Self::convert_account(&app_user_account), Error::<T>::NotModelCreator);

            let mut members = <ExpertMembers<T>>::get(&key);

            match members.binary_search(&old_member) {
                // If the search succeeds, the caller is already a member, so just return
                Ok(index) => {
                    members.remove(index);
                    <ExpertMembers<T>>::insert(&key, members);
                    Self::deposit_event(RawEvent::MemberRemoved(old_member));
                    Ok(())
                },
                // If the search fails, the caller is not a member, so just return
                Err(_) => Err(Error::<T>::NotMember.into()),
            }
        }

        #[weight = 0]
        pub fn air_drop_new_user_benefit(origin, app_id: Vec<u8>, user_id: Vec<u8>,
            receiver: <T as frame_system::Trait>::AccountId, amount: BalanceOf<T>) -> DispatchResult {

            let who = ensure_signed(origin)?;
            let key = T::Hashing::hash_of(&(&app_id, &user_id));

            ensure!(!<NewAccountBenefitRecords<T>>::contains_key(&key), Error::<T>::BenefitAlreadyDropped);

            // make sure sender has enough fund
            let available = T::Currency::free_balance(&who);
            ensure!(available > amount, Error::<T>::NotEnoughFund);

            // start air drop
            let _ = T::Currency::transfer(&who, &receiver, amount, KeepAlive);

            // record it
            <NewAccountBenefitRecords<T>>::insert(&key, amount);

            Self::deposit_event(RawEvent::NewUserBenefitDrpped(receiver, amount));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn convert_account(origin: &AuthAccountId) -> T::AccountId {
        let tmp: [u8; 32] = origin.clone().into();
        T::AccountId::decode(&mut &tmp[..]).unwrap_or_default()
    }

    pub fn is_platform_expert(who: &T::AccountId, app_id: &Vec<u8>) -> bool {
        let members = <AppPlatformExpertMembers<T>>::get(app_id);
        match members.binary_search(who) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn is_model_expert(who: &T::AccountId, app_id: &Vec<u8>, model_id: &Vec<u8>) -> bool {
        let key = T::Hashing::hash_of(&(app_id, model_id));
        let members = <ExpertMembers<T>>::get(&key);
        match members.binary_search(who) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    pub fn is_model_creator(who: &T::AccountId, app_id: &Vec<u8>, model_id: &Vec<u8>) -> bool {
        let key = T::Hashing::hash_of(&(app_id, model_id));
        <ModelCreators<T>>::contains_key(&key) && <ModelCreators<T>>::get(&key) == *who
    }
}

impl<T: Trait> AccountSet for Module<T> {
    type AccountId = T::AccountId;

    fn accounts() -> BTreeSet<T::AccountId> {
        Self::council_members().into_iter().collect::<BTreeSet<_>>()
    }
}

impl<T: Trait> Membership<T::AccountId, T::Hash> for Module<T> {
    fn is_platform(who: &T::AccountId, app_id: &Vec<u8>) -> bool {
        Self::is_platform_expert(who, app_id)
    }
    fn is_expert(who: &T::AccountId, app_id: &Vec<u8>, model_id: &Vec<u8>) -> bool {
        Self::is_model_expert(who, app_id, model_id)
    }

    fn set_model_creator(key: &T::Hash, creator: &T::AccountId, admin: &T::AccountId) -> () {
        // this interface is only available form pallet internal (from kp to member invoking)
        <ModelCreators<T>>::insert(key, creator);
        // give benifit to creator
        // TODO: should from Treasury
        let _ = T::Currency::transfer(
            admin,
            creator,
            T::ModelCreatorCreateBenefit::get(),
            KeepAlive,
        );
    }
}
