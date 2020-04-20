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

use crate::DispatchError;

use derive_try_from_primitive::TryFromPrimitive;
use std::convert::{TryFrom, TryInto};
use thiserror::Error as ThisError;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ThisError, TryFromPrimitive)]
#[repr(u8)]
/// Errors describing failed Registry transactions.
pub enum RegistryError {
    #[error("the provided checkpoint does not exist")]
    InexistentCheckpointId = 0,

    #[error("a registered project must have an initial checkpoint")]
    InexistentInitialProjectCheckpoint,

    #[error("the provided org does not exist")]
    InexistentOrg,

    #[error("the provided project does not exist")]
    InexistentProjectId,

    #[error("the provided user does not exist")]
    InexistentUser,

    #[error("an org with the same ID already exists")]
    DuplicateOrgId,

    #[error("a project with the same ID already exists")]
    DuplicateProjectId,

    #[error("a user with the same ID already exists.")]
    DuplicateUserId,

    #[error("the provided fee is insufficient")]
    InsufficientFee,

    #[error("the sender is not a project member")]
    InsufficientSenderPermissions,

    #[error("the provided checkpoint is not a descendant of the project's initial checkpoint")]
    InvalidCheckpointAncestry,

    #[error("the provided user is not eligible for unregistration")]
    UnregisterableUser,

    #[error("the provided org is not elibile for unregistration")]
    UnregisterableOrg,

    #[error("the account is already associated with a user")]
    UserAccountAssociated,
}

// The index with which the registry runtime module is declared
// in the Radicle Registry runtime - see the `construct_runtime`
// declaration in the `runtime` crate.
const REGISTRY_ERROR_INDEX: u8 = 7;

impl From<RegistryError> for DispatchError {
    fn from(error: RegistryError) -> Self {
        DispatchError::Module {
            index: REGISTRY_ERROR_INDEX,
            error: error as u8,
            message: None,
        }
    }
}

impl TryFrom<DispatchError> for RegistryError {
    type Error = &'static str;

    fn try_from(dispatch_error: DispatchError) -> Result<RegistryError, Self::Error> {
        if let DispatchError::Module {
            index,
            error,
            message: _,
        } = dispatch_error
        {
            if index == REGISTRY_ERROR_INDEX {
                return error.try_into().map_err(|_| {
                    "Failed to build the RegistryError variant specified in the DispatchError"
                });
            }
        }

        Err("The given DispatchError does not wrap a RegistryError.")
    }
}
