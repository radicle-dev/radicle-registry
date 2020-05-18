// Radicle Registry
// Copyright (C) 2019 Monadic GmbH <radicle@monadic.xyz>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as
// published by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! The Substrate Node Template runtime. This can be compiled with `#[no_std]`, ready for Wasm.

// We allow two clippy lints because the `impl_runtime_apis` and `construct_runtime` macros produce
// code that would fail otherwise.
#![allow(clippy::not_unsafe_ptr_arg_deref, clippy::string_lit_as_bytes)]
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]
#![feature(alloc_prelude)]

#[cfg(all(feature = "std", feature = "no-std"))]
std::compile_error!("Features \"std\" and \"no-std\" cannot be enabled simultaneously. Maybe a dependency implicitly enabled the \"std\" feature.");

extern crate alloc;

use frame_support::weights::Weight;
use frame_support::{construct_runtime, parameter_types};
use sp_core::ed25519;
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
use sp_runtime::{create_runtime_str, generic, Perbill};
use sp_std::prelude::*;
pub use sp_version::RuntimeVersion;

pub mod fees;
pub mod registry;
pub mod runtime_api;

pub use frame_system as system;
pub use pallet_balances as balances;
pub use radicle_registry_core::*;
pub use runtime_api::{api, RuntimeApi};

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = ed25519::Signature;

/// A hash of some data used by the chain.
///
/// Same as  [sp_runtime::traits::Hash::Output] for [Hashing].
pub type Hash = sp_core::H256;

pub type EventRecord = frame_system::EventRecord<Event, Hash>;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    crate::fees::PayTxFee,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<AccountId, Call, Signature, SignedExtra>;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("radicle-registry"),
    impl_name: create_runtime_str!("radicle-registry"),
    spec_version: 4,
    impl_version: 4,
    apis: runtime_api::VERSIONS,
    // Ignored by us. Only `spec_version` and `impl_version` are relevant.
    authoring_version: 3,
};

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
    sp_version::NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

parameter_types! {
    pub const BlockHashCount: BlockNumber = 250;
    pub const MaximumBlockWeight: Weight = 1_000_000;
    pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    pub const MaximumBlockLength: u32 = 5 * 1024 * 1024;
    pub const Version: RuntimeVersion = VERSION;
}

impl frame_system::Trait for Runtime {
    /// The identifier used to distinguish between accounts.
    type AccountId = AccountId;
    /// The aggregated dispatch type that is available for extrinsics.
    type Call = Call;
    /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
    type Lookup = sp_runtime::traits::IdentityLookup<AccountId>;
    /// The index type for storing how many extrinsics an account has signed.
    type Index = state::AccountTransactionIndex;
    /// The index type for blocks.
    type BlockNumber = BlockNumber;
    /// The type for hashing blocks and tries.
    type Hash = Hash;
    /// The hashing algorithm used.
    type Hashing = Hashing;
    /// The header type.
    type Header = Header;
    /// The ubiquitous event type.
    type Event = Event;
    /// The ubiquitous origin type.
    type Origin = Origin;
    /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
    type BlockHashCount = BlockHashCount;
    /// Maximum weight of each block. With a default weight system of 1byte == 1weight, 4mb is ok.
    type MaximumBlockWeight = MaximumBlockWeight;
    /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
    type MaximumBlockLength = MaximumBlockLength;
    /// Portion of the block weight that is available to all normal transactions.
    type AvailableBlockRatio = AvailableBlockRatio;
    /// Version of the runtime.
    type Version = Version;
    /// Converts a module to the index of the module in `construct_runtime!`.
    ///
    /// This type is being generated by `construct_runtime!`.
    type ModuleToIndex = ModuleToIndex;
    /// What to do if a new account is created.
    type OnNewAccount = ();
    /// What to do if an account is fully reaped from the system.
    type OnKilledAccount = Balances;
    /// The data to be stored in an account.
    type AccountData = balances::AccountData<Balance>;
}

parameter_types! {
    /// Minimum time between blocks in milliseconds
    pub const MinimumPeriod: u64 = 300;
}

impl pallet_timestamp::Trait for Runtime {
    /// A timestamp: milliseconds since the unix epoch.
    type Moment = u64;
    type OnTimestampSet = ();
    type MinimumPeriod = MinimumPeriod;
}

parameter_types! {
    /// The minimum amount required to keep an account open.
    /// Transfers leaving the recipient with less than this
    /// value fail.
    pub const ExistentialDeposit: u128 = 1;
    pub const TransferFee: u128 = 0;
    pub const CreationFee: u128 = 0;
    pub const TransactionBaseFee: u128 = 0;
    pub const TransactionByteFee: u128 = 0;
}

impl pallet_balances::Trait for Runtime {
    type Balance = Balance;
    type Event = Event;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
}

impl pallet_sudo::Trait for Runtime {
    type Event = Event;
    type Call = Call;
}

impl registry::Trait for Runtime {
    type Event = Event;
}

construct_runtime!(
        pub enum Runtime where
                Block = Block,
                NodeBlock = Block,
                UncheckedExtrinsic = UncheckedExtrinsic
        {
                System: system::{Module, Call, Storage, Config, Event<T>},
                Timestamp: pallet_timestamp::{Module, Call, Storage, Inherent},
                RandomnessCollectiveFlip: pallet_randomness_collective_flip::{Module, Call, Storage},
                Balances: pallet_balances::{Module, Call, Storage, Config<T>, Event<T>},
                Sudo: pallet_sudo::{Module, Call, Config<T>, Storage, Event<T>},
                Registry: registry::{Module, Call, Storage, Event, Inherent},
        }
);
