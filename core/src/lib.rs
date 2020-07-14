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

//! Basic types used in the Radicle Registry.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use parity_scale_codec::{Decode, Encode};
use sp_core::{ed25519, H256};
use sp_runtime::traits::BlakeTwo256;

pub use sp_runtime::DispatchError;

pub mod message;
pub mod state;

pub mod bytes128;
pub use bytes128::Bytes128;

mod id;
pub use id::{Id, InvalidIdError};

mod project_name;
pub use project_name::{InvalidProjectNameError, ProjectName};

mod error;
pub use error::{RegistryError, TransactionError};

/// The hashing algorightm to use
pub type Hashing = BlakeTwo256;

/// Identifier for accounts, an Ed25519 public key.
///
/// Each account has an associated [state::AccountBalance] and [state::AccountTransactionIndex].
pub type AccountId = ed25519::Public;

/// Amout of currency denominated in μRAD.
///
/// The non-negative balance of anything storing the amount of currency.
/// It can be used to represent the value of anything describing an amount,
/// e.g. an account balance, the value of a fee, etc.
pub type Balance = u128;

/// Convert amount of RAD into balance denominated in μRAD.
pub const fn rad_to_balance(rad: u64) -> Balance {
    rad as u128 * 1_000_000
}

/// The id of a project. Used as storage key.
pub type ProjectId = (ProjectName, ProjectDomain);

/// The domain under which a [crate::state::Projects1Data] lives.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
pub enum ProjectDomain {
    Org(Id),
    User(Id),
}

impl ProjectDomain {
    pub fn id(&self) -> Id {
        match self {
            Self::Org(id) | Self::User(id) => id.clone(),
        }
    }
}

pub type CheckpointId = H256;

#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "lowercase"))]
/// The Status of an org or user Id
pub enum IdStatus {
    /// The id is available and can be claimed
    Available,

    /// The id is curently taken by a user or by an org
    Taken,

    /// The id has been unregistered and is now retired
    Retired,
}
