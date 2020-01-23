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

use alloc::prelude::v1::*;

use parity_scale_codec::{Decode, Encode};
use sp_core::{ed25519, H256};
use sp_runtime::traits::BlakeTwo256;

pub use sp_runtime::DispatchError;

pub mod message;

mod bytes128;
pub use bytes128::Bytes128;

mod string32;
pub use string32::String32;

mod project_domain;
pub use project_domain::ProjectDomain;

/// Index of a transaction in the chain.
pub type Index = u32;

/// The hashing algorightm to use
pub type Hashing = BlakeTwo256;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = ed25519::Public;

/// Balance of an account.
pub type Balance = u128;

/// # Registry types

/// The name a project is registered with.
pub type ProjectName = String32;

pub type ProjectId = (ProjectName, ProjectDomain);

pub type CheckpointId = H256;

/// A project's version. Used in checkpointing.
pub type Version = String;

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub parent: Option<CheckpointId>,
    pub hash: H256,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub id: ProjectId,
    pub account_id: AccountId,
    pub members: Vec<AccountId>,
    pub current_cp: CheckpointId,
    pub metadata: Bytes128,
}
