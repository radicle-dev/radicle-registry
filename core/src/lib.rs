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
#![feature(alloc_prelude)]

extern crate alloc;

use sp_core::{ed25519, H256};
use sp_runtime::traits::BlakeTwo256;

pub use sp_runtime::DispatchError;

pub mod message;
pub mod state;

pub mod bytes128;
pub use bytes128::Bytes128;

pub mod string32;
pub use string32::String32;

mod project_domain;
pub use project_domain::ProjectDomain;

mod error;
pub use error::RegistryError;

/// The hashing algorightm to use
pub type Hashing = BlakeTwo256;

/// Identifier for accounts, an Ed25519 public key.
///
/// Each account has an associated [message::AccountBalance] and [message::Index].
pub type AccountId = ed25519::Public;

/// Balance of an account.
pub type Balance = u128;

/// The name a project is registered with.
pub type ProjectName = String32;

pub type ProjectId = (ProjectName, ProjectDomain);

pub type CheckpointId = H256;
