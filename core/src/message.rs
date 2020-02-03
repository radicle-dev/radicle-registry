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

use crate::{AccountId, Balance, Bytes128, CheckpointId, ProjectId, H256};
use parity_scale_codec::{Decode, Encode};

/// Registers a project on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, a new [crate::state::Project] with the given properties is added to the state.
///
/// [crate::state::Project::members] is initialized with the transaction author as the only member.
///
/// [crate::state::Project::account_id] is generated randomly.
///
/// # State-dependent validations
///
/// A project with the same ID must not yet exist.
///
/// A checkpoint with the given ID must exist.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterProject {
    pub id: ProjectId,

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
/// If successful, adds a new [crate::state::Checkpoint] with the given parameters to the state.
///
/// # State-dependent validations
///
/// If `previous_checkpoint_id` is provided a checkpoint with the given ID must exist in the state.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct CreateCheckpoint {
    pub project_hash: H256,
    pub previous_checkpoint_id: Option<CheckpointId>,
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
/// The transaction author must be in [crate::state::Project::members] of the given project.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct SetCheckpoint {
    pub project_id: ProjectId,
    pub new_checkpoint_id: CheckpointId,
}

/// Transfer funds from a project account to an account
///
/// # State changes
///
/// If successful, `balance` is deducated from the project account and added to the the recipient
/// account. The project account is given by [crate::state::Project::account_id] of the given project.
///
/// If the recipient account did not exist before, it is created. The recipient account may be a
/// user account or a project account.
///
/// # State-dependent validations
///
/// The author must be a member of [crate::state::Project::members].
///
/// The project account must have a balance of at least `balance`.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
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
