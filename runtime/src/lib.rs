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
#![allow(
    clippy::not_unsafe_ptr_arg_deref,
    clippy::string_lit_as_bytes,
    clippy::unnecessary_mut_passed
)]
#![cfg_attr(not(feature = "std"), no_std)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "256"]

#[cfg(all(feature = "std", feature = "no-std"))]
std::compile_error!("Features \"std\" and \"no-std\" cannot be enabled simultaneously. Maybe a dependency implicitly enabled the \"std\" feature.");

extern crate alloc;

use sp_core::ed25519;
use sp_runtime::traits::BlakeTwo256;
use sp_runtime::{create_runtime_str, generic};
pub use sp_version::RuntimeVersion;

pub use radicle_registry_core::*;
pub use runtime::api as runtime_api;
pub use runtime::api::{api, RuntimeApi};
pub use runtime::{Call, Event, Origin, Runtime};

pub mod fees;
pub mod registry;
mod runtime;
pub mod timestamp_in_digest;

pub use registry::DecodeKey;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = ed25519::Signature;

/// A hash of some data used by the chain.
///
/// Same as  [sp_runtime::traits::Hash::Output] for [Hashing].
pub type Hash = sp_core::H256;

/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;

/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
    frame_system::CheckTxVersion<Runtime>,
    frame_system::CheckGenesis<Runtime>,
    frame_system::CheckEra<Runtime>,
    frame_system::CheckNonce<Runtime>,
    frame_system::CheckWeight<Runtime>,
    crate::fees::PayTxFee,
);

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic = generic::UncheckedExtrinsic<AccountId, Call, Signature, SignedExtra>;

/// A timestamp: milliseconds since the unix epoch.
type Moment = u64;

pub const SPEC_VERSION: u32 = 16;

/// This runtime version.
pub const VERSION: RuntimeVersion = RuntimeVersion {
    spec_name: create_runtime_str!("radicle-registry"),
    impl_name: create_runtime_str!("radicle-registry"),
    spec_version: SPEC_VERSION,
    transaction_version: SPEC_VERSION,
    impl_version: 1,
    apis: runtime::api::VERSIONS,
    // Ignored by us. Only `spec_version` and `impl_version` are relevant.
    authoring_version: 3,
};

#[test]
fn crate_versions() {
    assert_eq!(
        env!("CARGO_PKG_VERSION_MINOR"),
        format!("{}", SPEC_VERSION),
        "Runtime spec_version does not match crate minor version"
    );
    assert_eq!(
        env!("CARGO_PKG_VERSION_PATCH"),
        format!("{}", VERSION.impl_version),
        "Runtime impl_version does not match crate patch version"
    );
}

/// The version infromation used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> sp_version::NativeVersion {
    sp_version::NativeVersion {
        runtime_version: VERSION,
        can_author_with: Default::default(),
    }
}

pub mod store {
    pub use crate::registry::store::*;
    pub type Account = frame_system::Account<crate::Runtime>;
    #[doc(inline)]
    pub use crate::registry::DecodeKey;
}

pub mod event {
    pub use crate::runtime::Event;
    pub type Record = frame_system::EventRecord<crate::runtime::Event, crate::Hash>;
    pub type Registry = crate::registry::Event;
    pub type System = frame_system::Event<crate::Runtime>;

    /// Return the index of the transaction in the block that dispatched the event.
    ///
    /// Returns `None` if the event was not dispatched as part of a transaction.
    #[cfg(feature = "std")]
    pub fn transaction_index(record: &Record) -> Option<u32> {
        match record.phase {
            frame_system::Phase::ApplyExtrinsic(i) => Some(i),
            _ => None,
        }
    }
}

pub mod call {
    pub type Registry = crate::registry::Call<crate::Runtime>;
    pub type System = frame_system::Call<crate::Runtime>;
    pub type Sudo = pallet_sudo::Call<crate::Runtime>;
}

#[cfg(feature = "std")]
pub mod genesis {
    pub use crate::runtime::{BalancesConfig, GenesisConfig, SudoConfig, SystemConfig};
}
