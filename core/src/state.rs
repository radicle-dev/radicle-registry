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

//! Type definitions for all entities stored in the ledger state.

use alloc::vec::Vec;
use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::Hash;

use crate::{AccountId, Balance, Bytes128, CheckpointId, Hashing, Id, ProjectName, H256};

/// A checkpoint defines an immutable state of a project’s off-chain data via a hash.
///
/// Checkpoints are used by [Project::current_cp]
///
/// Checkpoints are identified by their content hash. See [Checkpoint::id].
///
/// # Storage
///
/// Checkpoints are stored as a map using [Checkpoint::id] to derive the key.
///
/// # Invariants
///
/// * If `parent` is [Some] then the referenced checkpoint exists in the state.
///
/// # Relevant messages
///
/// * [crate::message::CreateCheckpoint]
/// * [crate::message::SetCheckpoint]
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    /// Previous checkpoint in the project histor.
    pub parent: Option<CheckpointId>,
    /// Hash that identifies a project’s off-chain data.
    pub hash: H256,
}

impl Checkpoint {
    pub fn id(&self) -> CheckpointId {
        Hashing::hash_of(&self)
    }
}

/// # Storage
///
/// This type is only used for storage. See [crate::Project] for the
/// complete Project type to be used everywhere else.
///
/// Projects are stored as a map with the key derived from a given [crate::ProjectId].
/// The project ID can be extracted from the storage key.
///
/// # Invariants
///
/// * `current_cp` is guaranteed to point to an existing [Checkpoint]
/// * `metadata` is immutable
///
/// # Relevant messages
///
/// * [crate::message::SetCheckpoint]
/// * [crate::message::RegisterProject]
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Project {
    /// Links to the checkpoint of project state.
    ///
    /// Updated with the [crate::message::SetCheckpoint] transaction.
    pub current_cp: CheckpointId,

    /// Opaque metadata that is controlled by the DApp.
    pub metadata: Bytes128,
}

/// Balance associated with an [crate::AccountId].
///
/// See the [Balances Pallet](https://substrate.dev/rustdocs/master/pallet_balances/index.html) for
/// more information.
///
/// # Storage
///
/// Balances are stored as a map with a key derived from [crate::AccountId].
///
/// # Relevant messages
///
/// * [crate::message::Transfer]
/// * [crate::message::TransferFromOrg]
pub type AccountBalance = Balance;

/// Next index (nonce) for a transaction of an account.
///
/// The index for an [crate::AccountId] increases whenever a transaction by the account owner is
/// applied.
///
/// # Storage
///
/// Indicies are stored as a map with a key derived from [crate::AccountId].
pub type AccountTransactionIndex = u32;

/// # Storage
///
/// Orgs are stored as a map with the key derived from [crate::Id].
/// The org ID can be extracted from the storage key.
///
/// # Relevant messages
///
/// * [crate::message::RegisterOrg]
/// * [crate::message::UnregisterOrg]
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub enum Orgs1Data {
    V1(OrgV1),
}

impl Orgs1Data {
    /// Creates new instance in the most up to date version
    pub fn new(account_id: AccountId, members: Vec<Id>, projects: Vec<ProjectName>) -> Self {
        Self::V1(OrgV1 {
            account_id,
            members,
            projects,
        })
    }

    /// Account ID that holds the org funds.
    ///
    /// It is randomly generated and, unlike for other accounts,
    /// there is no private key that controls this account.
    pub fn account_id(&self) -> AccountId {
        match self {
            Self::V1(org) => org.account_id,
        }
    }

    /// Set of members of the org. Members are allowed to manage
    /// the org, its projects, and transfer funds.
    ///
    /// It is initialized with the user id associated with the author
    /// of the [crate::message::RegisterOrg] transaction.
    /// It cannot be changed at the moment.
    pub fn members(&self) -> &Vec<Id> {
        match self {
            Self::V1(org) => &org.members,
        }
    }

    /// Set of all projects owned by the org. Members are allowed to register
    /// a project by sending a [crate::message::RegisterProject] transaction.
    pub fn projects(&self) -> &Vec<ProjectName> {
        match self {
            Self::V1(org) => &org.projects,
        }
    }

    /// Add the given project to the list of [Orgs1Data::projects].
    /// Return a new Org with the new project included or the
    /// same org if the org already contains that project.
    pub fn add_project(self, project_name: ProjectName) -> Self {
        match self {
            Self::V1(org) => Self::V1(org.add_project(project_name)),
        }
    }

    /// Add the given user to the list of [Orgs1Data::members].
    /// Return a new Org with the new member included or the
    /// same org if the org already contains that member.
    pub fn add_member(self, user_id: Id) -> Self {
        match self {
            Self::V1(org) => Self::V1(org.add_member(user_id)),
        }
    }
}

/// # Invariants
///
/// * `account_id` is immutable
/// * `projects` is a set of all the projects owned by the Org.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct OrgV1 {
    /// Account ID that holds the org funds.
    ///
    /// It is randomly generated and, unlike for other accounts,
    /// there is no private key that controls this account.
    pub account_id: AccountId,

    /// Set of members of the org. Members are allowed to manage
    /// the org, its projects, and transfer funds.
    ///
    /// It is initialized with the user id associated with the author
    /// of the [crate::message::RegisterOrg] transaction.
    /// It cannot be changed at the moment.
    pub members: Vec<Id>,

    /// Set of all projects owned by the org. Members are allowed to register
    /// a project by sending a [crate::message::RegisterProject] transaction.
    pub projects: Vec<ProjectName>,
}

impl OrgV1 {
    /// Add the given project to the list of [OrgV1::projects].
    /// Return a new Org with the new project included or the
    /// same org if the org already contains that project.
    pub fn add_project(mut self, project_name: ProjectName) -> Self {
        if !self.projects.contains(&project_name) {
            self.projects.push(project_name);
        }
        self
    }

    /// Add the given user to the list of [OrgV1::members].
    /// Return a new Org with the new member included or the
    /// same org if the org already contains that member.
    pub fn add_member(mut self, user_id: Id) -> Self {
        if !self.members.contains(&user_id) {
            self.members.push(user_id);
        }
        self
    }
}

/// # Storage
///
/// This type is only used for storage. See [crate::User] for the
/// complete User type to be used everywhere else.
///
/// Users are stored as a map with the key derived from [crate::User::id].
/// The user ID can be extracted from the storage key.
///
/// # Invariants
///
/// * `account_id` is immutable
/// * `projects` is a set of all the projects owned by the User.
///
/// # Relevant messages
///
/// * [crate::message::RegisterUser]
/// * [crate::message::UnregisterUser]
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq)]
pub struct User {
    /// Account ID that holds the user funds.
    pub account_id: AccountId,

    /// Set of all projects owned by the user.
    pub projects: Vec<ProjectName>,
}

impl User {
    pub fn add_project(mut self, project_name: ProjectName) -> User {
        if !self.projects.contains(&project_name) {
            self.projects.push(project_name);
        }
        self
    }
}
