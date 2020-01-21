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

//! Transaction related types used in the Radicle Registry.

extern crate alloc;

use crate::{AccountId, Balance, Bytes128, CheckpointId, ProjectId};
use parity_scale_codec::{Decode, Encode};
use sp_core::H256;

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterProject {
    pub id: ProjectId,
    pub checkpoint_id: CheckpointId,
    pub metadata: Bytes128,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct CreateCheckpoint {
    pub project_hash: H256,
    pub previous_checkpoint_id: Option<CheckpointId>,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct SetCheckpoint {
    pub project_id: ProjectId,
    pub new_checkpoint_id: CheckpointId,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct TransferFromProject {
    pub project: ProjectId,
    pub recipient: AccountId,
    pub value: Balance,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Transfer {
    pub recipient: AccountId,
    pub balance: Balance,
}
