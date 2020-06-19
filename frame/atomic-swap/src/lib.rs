// This file is part of Substrate.

// Copyright (C) 2017-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Atomic swap support pallet

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]

mod tests;

use sp_std::{prelude::*, marker::PhantomData, ops::{Deref, DerefMut}};
use sp_io::hashing::blake2_256;
use frame_support::{
	Parameter, decl_module, decl_storage, decl_event, decl_error, ensure,
	traits::{Get, Currency, ReservableCurrency, BalanceStatus},
	weights::Weight,
	dispatch::DispatchResult,
};
use frame_system::{self as system, ensure_signed};
use codec::{Encode, Decode};
use sp_runtime::RuntimeDebug;

/// Pending atomic swap operation.
#[derive(Clone, Eq, PartialEq, RuntimeDebug, Encode, Decode)]
pub struct PendingSwap<T: Trait> {
	/// Source of the swap.
	pub source: AccountIdFor<T>,
	/// Action of this swap.
	pub action: T::SwapAction,
	/// End block of the lock.
	pub end_block: BlockNumberFor<T>,
}

/// AccountId type from the pallet's point of view.
pub type AccountIdFor<T> = <T as frame_system::Trait>::AccountId;

/// BlockNumber type from the pallet's point of view.
pub type BlockNumberFor<T> = <T as frame_system::Trait>::BlockNumber;

/// Hashed proof type.
pub type HashedProof = [u8; 32];

/// Definition of a pending atomic swap action. It contains the following three phrases:
///
/// - **Reserve**: reserve the resources needed for a swap. This is to make sure that **Claim**
/// succeeds with best efforts.
/// - **Claim**: claim any resources reserved in the first phrase.
/// - **Cancel**: cancel any resources reserved in the first phrase.
pub trait SwapAction<T: Trait> {
	/// Reserve the resources needed for the swap, from the given `source`. The reservation is
	/// allowed to fail. If that is the case, the the full swap creation operation is cancelled.
	fn reserve(&self, source: &AccountIdFor<T>) -> DispatchResult;
	/// Claim the reserved resources, with `source` and `target`. Returns whether the claim
	/// succeeds.
	fn claim(&self, source: &AccountIdFor<T>, target: &AccountIdFor<T>) -> bool;
	/// Weight for executing the operation.
	fn weight(&self) -> Weight;
	/// Cancel the resources reserved in `source`.
	fn cancel(&self, source: &AccountIdFor<T>);
}

/// A swap action that only allows transferring balances.
#[derive(Clone, RuntimeDebug, Eq, PartialEq, Encode, Decode)]
pub struct BalanceSwapAction<T: Trait, C: ReservableCurrency<T::AccountId>> {
	value: <C as Currency<<T as frame_system::Trait>::AccountId>>::Balance,
	_marker: PhantomData<C>,
}

impl<T: Trait, C> BalanceSwapAction<T, C> where
	C: ReservableCurrency<T::AccountId>,
{
	/// Create a new swap action value of balance.
	pub fn new(value: <C as Currency<<T as frame_system::Trait>::AccountId>>::Balance) -> Self {
		Self { value, _marker: PhantomData }
	}
}

impl<T: Trait, C> Deref for BalanceSwapAction<T, C> where
	C: ReservableCurrency<T::AccountId>,
{
	type Target = <C as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

impl<T: Trait, C> DerefMut for BalanceSwapAction<T, C> where
	C: ReservableCurrency<T::AccountId>,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.value
	}
}

impl<T: Trait, C> SwapAction<T> for BalanceSwapAction<T, C> where
	C: ReservableCurrency<T::AccountId>,
{
	fn reserve(&self, source: &AccountIdFor<T>) -> DispatchResult {
		C::reserve(&source, self.value)
	}

	fn claim(&self, source: &AccountIdFor<T>, target: &AccountIdFor<T>) -> bool {
		C::repatriate_reserved(source, target, self.value, BalanceStatus::Free).is_ok()
	}

	fn weight(&self) -> Weight {
		T::DbWeight::get().reads_writes(1, 1)
	}

	fn cancel(&self, source: &AccountIdFor<T>) {
		C::unreserve(source, self.value);
	}
}

/// Atomic swap's pallet configuration trait.
pub trait Trait: frame_system::Trait {
	/// The overarching event type.
	type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
	/// Swap action.
	type SwapAction: SwapAction<Self> + Parameter;
	/// Limit of proof size.
	///
	/// Atomic swap is only atomic if once the proof is revealed, both parties can submit the proofs
	/// on-chain. If A is the one that generates the proof, then it requires that either:
	/// - A's blockchain has the same proof length limit as B's blockchain.
	/// - Or A's blockchain has shorter proof length limit as B's blockchain.
	///
	/// If B sees A is on a blockchain with larger proof length limit, then it should kindly refuse
	/// to accept the atomic swap request if A generates the proof, and asks that B generates the
	/// proof instead.
	type ProofLimit: Get<u32>;
}

decl_storage! {
	trait Store for Module<T: Trait> as AtomicSwap {
		pub PendingSwaps: double_map
			hasher(twox_64_concat) T::AccountId, hasher(blake2_128_concat) HashedProof
			=> Option<PendingSwap<T>>;
	}
}

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Swap already exists.
		AlreadyExist,
		/// Swap proof is invalid.
		InvalidProof,
		/// Proof is too large.
		ProofTooLarge,
		/// Source does not match.
		SourceMismatch,
		/// Swap has already been claimed.
		AlreadyClaimed,
		/// Swap does not exist.
		NotExist,
		/// Duration has not yet passed for the swap to be cancelled.
		DurationNotPassed,
	}
}

decl_event!(
	/// Event of atomic swap pallet.
	pub enum Event<T> where
		AccountId = AccountIdFor<T>,
		PendingSwap = PendingSwap<T>,
	{
		/// Swap created.
		NewSwap(AccountId, HashedProof, PendingSwap),
		/// Swap claimed. The last parameter indicates whether the execution succeeds.
		SwapClaimed(AccountId, HashedProof, bool),
		/// Swap cancelled.
		SwapCancelled(AccountId, HashedProof),
	}
);

decl_module! {
	/// Module definition of atomic swap pallet.
	pub struct Module<T: Trait> for enum Call where origin: T::Origin {
		type Error = Error<T>;

		fn deposit_event() = default;

		/// Register a new atomic swap, declaring an intention to send funds from origin to target
		/// on the current blockchain. The target can claim the fund using the revealed proof. If
		/// the fund is not claimed after `duration` blocks, then the sender can cancel the swap.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `target`: Receiver of the atomic swap.
		/// - `hashed_proof`: The blake2_256 hash of the secret proof.
		/// - `balance`: Funds to be sent from origin.
		/// - `duration`: Locked duration of the atomic swap. For safety reasons, it is recommended
		///   that the revealer uses a shorter duration than the counterparty, to prevent the
		///   situation where the revealer reveals the proof too late around the end block.
		#[weight = T::DbWeight::get().reads_writes(1, 1).saturating_add(40_000_000)]
		fn create_swap(
			origin,
			target: AccountIdFor<T>,
			hashed_proof: HashedProof,
			action: T::SwapAction,
			duration: BlockNumberFor<T>,
		) {
			let source = ensure_signed(origin)?;
			ensure!(
				!PendingSwaps::<T>::contains_key(&target, hashed_proof),
				Error::<T>::AlreadyExist
			);

			action.reserve(&source)?;

			let swap = PendingSwap {
				source,
				action,
				end_block: frame_system::Module::<T>::block_number() + duration,
			};
			PendingSwaps::<T>::insert(target.clone(), hashed_proof.clone(), swap.clone());

			Self::deposit_event(
				RawEvent::NewSwap(target, hashed_proof, swap)
			);
		}

		/// Claim an atomic swap.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `proof`: Revealed proof of the claim.
		#[weight = T::DbWeight::get().reads_writes(2, 2)
		  .saturating_add(40_000_000)
		  .saturating_add((proof.len() as Weight).saturating_mul(100))
		]
		fn claim_swap(
			origin,
			proof: Vec<u8>,
		) -> DispatchResult {
			ensure!(
				proof.len() <= T::ProofLimit::get() as usize,
				Error::<T>::ProofTooLarge,
			);

			let target = ensure_signed(origin)?;
			let hashed_proof = blake2_256(&proof);

			let swap = PendingSwaps::<T>::get(&target, hashed_proof)
				.ok_or(Error::<T>::InvalidProof)?;

			let succeeded = swap.action.claim(&swap.source, &target);

			PendingSwaps::<T>::remove(target.clone(), hashed_proof.clone());

			Self::deposit_event(
				RawEvent::SwapClaimed(target, hashed_proof, succeeded)
			);

			Ok(())
		}

		/// Cancel an atomic swap. Only possible after the originally set duration has passed.
		///
		/// The dispatch origin for this call must be _Signed_.
		///
		/// - `target`: Target of the original atomic swap.
		/// - `hashed_proof`: Hashed proof of the original atomic swap.
		#[weight = T::DbWeight::get().reads_writes(1, 1).saturating_add(40_000_000)]
		fn cancel_swap(
			origin,
			target: AccountIdFor<T>,
			hashed_proof: HashedProof,
		) {
			let source = ensure_signed(origin)?;

			let swap = PendingSwaps::<T>::get(&target, hashed_proof)
				.ok_or(Error::<T>::NotExist)?;
			ensure!(
				swap.source == source,
				Error::<T>::SourceMismatch,
			);
			ensure!(
				frame_system::Module::<T>::block_number() >= swap.end_block,
				Error::<T>::DurationNotPassed,
			);

			swap.action.cancel(&swap.source);
			PendingSwaps::<T>::remove(&target, hashed_proof.clone());

			Self::deposit_event(
				RawEvent::SwapCancelled(target, hashed_proof)
			);
		}
	}
}
