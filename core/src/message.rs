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

use crate::{AccountId, Balance, Bytes128, CheckpointId, OrgId, ProjectId};
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;

/// Registers a project on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, a new [crate::Project] with the given properties is added to the state.
///
/// [crate::Project::members] is initialized with the transaction author as the only member.
///
/// # State-dependent validations
///
/// A project with the same ID must not yet exist within the same org.
///
/// A checkpoint with the given ID must exist.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterProject {
    // A unique project id within the org to be registered.
    pub id: ProjectId,

    // The id of the Org under which to register this project.
    pub org_id: OrgId,

    /// Initial checkpoint of the project.
    pub checkpoint_id: CheckpointId,

    /// Opaque metadata that cannot be changed.
    ///
    /// Used by the application.
    pub metadata: Bytes128,
}

/// Add a new checkpoint to the state.
///
/// # State changes
///
/// If successful, adds a new [crate::Checkpoint] with the given parameters to the state.
///
/// # State-dependent validations
///
/// If `previous_checkpoint_id` is provided a checkpoint with the given ID must exist in the state.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct CreateCheckpoint {
    pub project_hash: H256,
    pub previous_checkpoint_id: Option<CheckpointId>,
}

/// Updates [crate::Project::current_cp].
///
/// # State changes
///
/// If successful, adds a new [crate::Checkpoint] with the given parameters to the state.
///
/// # State-dependent validations
///
/// The project `project_id` must exist.
///
/// The checkpoint `new_checkpoint_id` must exist.
///
/// The transaction author must be in [crate::Project::members] of the given project.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct SetCheckpoint {
    pub project_id: ProjectId,
    pub project_org_id: OrgId,
    pub new_checkpoint_id: CheckpointId,
}

/// Transfer funds from a project account to an account
///
/// # State changes
///
/// If successful, `balance` is deducated from the project account and added to the the recipient
/// account. The project account is given by [crate::Project::account_id] of the given project.
///
/// If the recipient account did not exist before, it is created. The recipient account may be a
/// user account or a project account.
///
/// # State-dependent validations
///
/// The author must be a member of [crate::Project::members].
///
/// The project account must have a balance of at least `balance`.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
//TODO(nuno) delete or move it to Org
pub struct TransferFromProject {
    pub project: ProjectId,
    pub recipient: AccountId,
    pub value: Balance,
}

/// Transfer funds from one account to another.
///
/// # State changes
///
/// If successful, `balance` is deducated from the transaction author account and added to the the
/// recipient account. If the recipient account did not exist before, it is created.
///
/// The recipient account may be a user account or a project account.
///
/// # State-dependent validations
///
/// The author account must have a balance of at least `balance`.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    pub recipient: AccountId,
    pub balance: Balance,
}
