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

use crate::{AccountId, Balance, Bytes128, CheckpointId, Id, ProjectDomain, ProjectName, H256};
use alloc::prelude::v1::Vec;
use parity_scale_codec::{Decode, Encode};

/// Registers an org on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, a new [crate::state::Orgs1Data] with the given properties is added to the state.
///
/// [crate::state::Orgs1Data::members] is initialized with the user id associated with the author
/// as the only member.
///
/// [crate::state::Orgs1Data::account_id] is generated randomly.
///
/// # State-dependent validations
///
/// An Org with the same ID must not yet exist.
///
/// A User with the same ID must not yet exist.
///
/// A user associated with the author must exist.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterOrg {
    pub org_id: Id,
}

/// Unregisters an org on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, the targeted Org is removed from the state.
///
/// # State-dependent validations
///
/// The targeted org must exist, have no projects, and a user
/// associated with the author must exist and be its only member.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct UnregisterOrg {
    pub org_id: Id,
}

/// Registers a user on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, a new [crate::state::Users1Data] with the given properties is added to the state.
///
/// [crate::state::Users1Data::account_id] is generated randomly.
///
/// # State-dependent validations
///
/// A user with the same ID must not yet exist.
///
/// An Org with the same ID must not yet exist.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterUser {
    pub user_id: Id,
}

/// Unregisters a user on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, the targeted User is removed from the state.
///
/// # State-dependent validations
///
/// The targeted user must exist and have no projects and the
/// the transaction origin must be the associated account.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct UnregisterUser {
    pub user_id: Id,
}

/// Register a new member for an org on the Registry with the given user ID.
///
/// # State changes
///
/// If successful, the `user_id` is added to [crate::state::Orgs1Data::members] of `org_id` .
///
/// # State-dependent validations
///
/// The identified org must exit.
///
/// The user associated with the author must be a member of the identified org.
///
/// A user associated with the `user_id` must exist.
///
/// The `user_id` must not already be a member of the org.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterMember {
    // The member to register, unique in the org.
    pub user_id: Id,

    /// The org in which to register the member.
    pub org_id: Id,
}

/// Register a project on the Radicle Registry with the given ID.
///
/// # State changes
///
/// If successful, a new [crate::state::Projects1Data] with the given
/// properties is added to the state.
///
///
/// # State-dependent validations
///
/// The involved project domain must exit.
///
/// A user associated with the author must exist.
///
/// The user associated with the author must a member of
/// the involved org, when an org is specified as the project domain.
///
/// A checkpoint with the given ID must exist.
///
/// A project with the same name must not yet exist in domain.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterProject {
    // The name of the project to register, unique under its domain.
    pub project_name: ProjectName,

    /// The domain of the project.
    pub project_domain: ProjectDomain,

    /// Initial checkpoint of the project.
    pub checkpoint_id: CheckpointId,

    /// Opaque and imutable metadata, used by the application.
    pub metadata: Bytes128,
}

/// Add a new checkpoint to the state.
///
/// # State changes
///
/// If successful, adds a new [crate::state::Checkpoints1Data] with the given parameters
/// to the state.
///
/// # State-dependent validations
///
/// If `previous_checkpoint_id` is provided a checkpoint with the given ID must exist in the state.
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct CreateCheckpoint {
    pub project_hash: H256,
    pub previous_checkpoint_id: Option<CheckpointId>,
}

/// Updates [crate::state::ProjectV1::current_cp].
///
/// # State changes
///
/// If successful, adds a new [crate::state::Checkpoints1Data] with the given parameters
/// to the state.
///
/// # State-dependent validations
///
/// The project `project_id` must exist.
///
/// The checkpoint `new_checkpoint_id` must exist.
///
/// A user associated with the transaction author must exist and
/// be a member of the Org of the given project.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct SetCheckpoint {
    pub project_name: ProjectName,
    pub project_domain: ProjectDomain,
    pub new_checkpoint_id: CheckpointId,
}

/// Transfer funds from an org account to an account.
///
/// # State changes
///
/// If successful, `amount` is deducated from the org account and
/// added to the the recipient account. The org account is given
/// by [crate::state::Orgs1Data::account_id] of the given org.
///
/// If the recipient account did not exist before, it is created.
/// The recipient account may be a user account or an org account.
///
/// # State-dependent validations
///
/// A user associated with the transaction author must exist and
/// be a member of the Org of the given project.
///
/// The org account must have a balance of at least `amount`.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct TransferFromOrg {
    pub org_id: Id,
    pub recipient: AccountId,
    pub amount: Balance,
}

/// Transfer funds from one account to another.
///
/// # State changes
///
/// If successful, `amount` is deducated from the transaction author
/// account and added to the the recipient account. If the recipient
/// account did not exist before, it is created.
///
/// The recipient account may be a user account or an org account.
///
/// # State-dependent validations
///
/// The author account must have a balance of at least `amount`.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    pub recipient: AccountId,
    pub amount: Balance,
}

/// Attempts to update the on-chain runtime with the new given one.
/// The `code` must be a valid WASM module and adhere to the substrate runtime API.
///
/// # State changes
///
/// If successful, the given `code` will be the new one herein.
///
/// # State-dependent validations
///
/// The tx author must be the chain's sudo key
///
/// The `spec_version` of the given runtime code needs to be greater than
/// the `spec_version` of the chain runtime.
///
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct UpdateRuntime {
    pub code: Vec<u8>,
}
