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

use crate::{AccountId, Balance, Bytes128, CheckpointId, Hashing, ProjectId, H256};

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
/// * [crate::message::SetCheckpiont]
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
/// Projects are stored as a map with the key derived from [Project::id]. The project ID can be
/// extracted from the storage key.
///
/// # Invariants
///
/// * `current_cp` is guaranteed to point to an existing [Checkpoint]
/// * `metadata` is immutable
/// * `account_id` is immutable
///
/// # Relevant messages
///
/// * [crate::message::SetCheckpoint]
/// * [crate::message::RegisterProject]
#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Project {
    /// ID of the project. The storage key is derived from this value.
    pub id: ProjectId,

    /// Account ID that holds the projecs founds.
    ///
    /// This is randomly generated and unlike for other accounts there is no private key that
    /// controls this account.
    pub account_id: AccountId,

    /// List of members that are allowed to modify the project and transfer funds.
    ///
    /// This is initialized with the author of the [crate::message::RegisterProject] transaction.
    /// It cannot be changed at the moment.
    pub members: Vec<AccountId>,

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
/// * [crate::message::TransferFromProject]
pub type AccountBalance = Balance;

/// Next index (nonce) for a transaction of an account.
///
/// The index for an [crate::AccountId] increases whenever a transaction by the account owner is
/// applied.
///
/// # Storage
///
/// Indicies are stored as a map with a key derived from [crate::AccountId].
pub type Index = u32;
