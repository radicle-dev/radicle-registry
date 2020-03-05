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

//! All transaction messages that can be submitted to the ledger
//!
//! See the README.md for more information on how to document messages.
extern crate alloc;

use crate::{AccountId, Balance, Bytes128, CheckpointId, OrgId, ProjectName, H256};
use parity_scale_codec::{Decode, Encode};

/// Registers an org on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, a new [crate::state::Org] with the given properties is added to the state.
///
/// [crate::state::Org::members] is initialized with the transaction author as the only member.
///
/// [crate::state::Org::account_id] is generated randomly.
///
/// # State-dependent validations
///
/// An Org with the same ID must not yet exist.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterOrg {
    pub org_id: OrgId,
    pub bid: Balance,
}

/// Unregisters an org on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, the targeted Org is removed from the state.
///
/// # State-dependent validations
///
/// The targeted org must exist and have no projects and the
/// the transaction origin must be its only member.
///
/// The provided bid must suffice the mandatory transaction
/// fees.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct UnregisterOrg {
    pub org_id: OrgId,
    pub bid: Balance,
}

/// Register a project on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, a new [crate::state::Project] with the given
/// properties is added to the state.
///
///
/// # State-dependent validations
///
/// The involved org must exit.
///
/// The author must be a member of the involved org.
///
/// A checkpoint with the given ID must exist.
///
/// A project with the same name must not yet exist in the org.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterProject {
    // The name of the project to register, unique in the org.
    pub project_name: ProjectName,

    /// The org in which to register the project.
    pub org_id: OrgId,

    /// Initial checkpoint of the project.
    pub checkpoint_id: CheckpointId,

    /// Opaque and imutable metadata, used by the application.
    pub metadata: Bytes128,
}

/// Add a new checkpoint to the state.
///
/// # State changes
///
/// If successful, adds a new [crate::state::Checkpoint] with the given parameters to the state.
///
/// # State-dependent validations
///
/// If `previous_checkpoint_id` is provided a checkpoint with the given ID must exist in the state.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct CreateCheckpoint {
    pub project_hash: H256,
    pub previous_checkpoint_id: Option<CheckpointId>,
    pub bid: Balance,
}

/// Updates [crate::state::Project::current_cp].
///
/// # State changes
///
/// If successful, adds a new [crate::state::Checkpoint] with the given parameters to the state.
///
/// # State-dependent validations
///
/// The project `project_id` must exist.
///
/// The checkpoint `new_checkpoint_id` must exist.
///
/// The transaction author must be part of the [crate::state::Org::members] of the given project.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct SetCheckpoint {
    pub project_name: ProjectName,
    pub org_id: OrgId,
    pub new_checkpoint_id: CheckpointId,
}

/// Transfer funds from an org account to an account.
///
/// # State changes
///
/// If successful, `value` is deducated from the org account and
/// added to the the recipient account. The org account is given
/// by [crate::state::Org::account_id] of the given org.
///
/// If the recipient account did not exist before, it is created.
/// The recipient account may be a user account or an org account.
///
/// # State-dependent validations
///
/// The author must be a member of [crate::state::Org::members].
///
/// The org account must have a balance of at least `value`.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct TransferFromOrg {
    pub org_id: OrgId,
    pub recipient: AccountId,
    pub value: Balance,
}

/// Transfer funds from one account to another.
///
/// # State changes
///
/// If successful, `balance` is deducated from the transaction author
/// account and added to the the recipient account. If the recipient
/// account did not exist before, it is created.
///
/// The recipient account may be a user account or an org account.
///
/// # State-dependent validations
///
/// The author account must have a balance of at least `balance`.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    pub recipient: AccountId,
    pub balance: Balance,
}
