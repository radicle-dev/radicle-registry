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

use alloc::vec::Vec;

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

/// The non-negative balance of anything storing the amount of currency.
/// It can be used to represent the value of anything describing an amount,
/// e.g. an account balance, the value of a fee, etc.
/// Represents a μRAD.
pub type Balance = u128;

/// The id of a project. Used as storage key.
pub type ProjectId = (ProjectName, ProjectDomain);

/// The domain under which a [Project] lives.
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

/// Org
///
/// Different from [state::Org] in which this type gathers
/// both the [`Id`] and the other [`Org`] fields, respectively stored
/// as an Org's storage key and data, into one complete type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Org {
    // Unique id of an Org.
    pub id: Id,

    /// See [state::Org::account_id]
    pub account_id: AccountId,

    /// See [state::Org::members]
    pub members: Vec<Id>,

    /// See [state::Org::projects]
    pub projects: Vec<ProjectName>,
}

impl Org {
    pub fn new(id: Id, org: state::Org) -> Org {
        Org {
            id,
            account_id: org.account_id,
            members: org.members,
            projects: org.projects,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Project {
    /// The name of the project, unique within its org.
    pub name: ProjectName,

    /// The domain to which the project belongs.
    pub domain: ProjectDomain,

    /// See [state::Project::current_cp]
    pub current_cp: CheckpointId,

    /// See [state::Project::metadata]
    pub metadata: Bytes128,
}

impl Project {
    /// Build a [crate::Project] given all its properties obtained from storage.
    pub fn new(name: ProjectName, domain: ProjectDomain, project: state::Project) -> Self {
        Project {
            name,
            domain,
            current_cp: project.current_cp,
            metadata: project.metadata,
        }
    }
}

pub type CheckpointId = H256;

/// User
///
/// Different from [state::User] in which this type gathers
/// both the [`Id`] and the other [`User`] fields, respectively stored
/// as an User's storage key and data, into one complete type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct User {
    // Unique id of a User.
    pub id: Id,

    /// See [state::User::account_id]
    pub account_id: AccountId,

    /// See [state::User::projects]
    pub projects: Vec<ProjectName>,
}

impl User {
    pub fn new(id: Id, user: state::User) -> User {
        User {
            id,
            account_id: user.account_id,
            projects: user.projects,
        }
    }
}
