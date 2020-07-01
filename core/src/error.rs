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

use core::convert::{TryFrom, TryInto};
use derive_try_from_primitive::TryFromPrimitive;

/// Error that may be the result of executing a transaction.
///
/// The error is either a [RegistryError] if it originated from our registry code or a
/// [DispatchError] from other substrate modules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
pub enum TransactionError {
    #[cfg_attr(feature = "std", error(transparent))]
    RegistryError(#[cfg_attr(feature = "std", from)] RegistryError),

    #[cfg_attr(feature = "std", error("{0:?}"))]
    OtherDispatchError(DispatchError),
}

impl From<DispatchError> for TransactionError {
    fn from(dispatch_error: DispatchError) -> Self {
        dispatch_error
            .try_into()
            .map(TransactionError::RegistryError)
            .unwrap_or(TransactionError::OtherDispatchError(dispatch_error))
    }
}

/// Errors describing failed Registry transactions.
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
#[cfg_attr(feature = "std", derive(thiserror::Error))]
#[repr(u8)]
pub enum RegistryError {
    #[cfg_attr(feature = "std", error("the provided checkpoint does not exist"))]
    InexistentCheckpointId = 0,

    #[cfg_attr(
        feature = "std",
        error("a registered project must have an initial checkpoint")
    )]
    InexistentInitialProjectCheckpoint = 1,

    #[cfg_attr(feature = "std", error("the provided org does not exist"))]
    InexistentOrg = 2,

    #[cfg_attr(feature = "std", error("the provided project does not exist"))]
    InexistentProjectId = 3,

    #[cfg_attr(feature = "std", error("the provided user does not exist"))]
    InexistentUser = 4,

    #[deprecated(note = "Superseded by IdAlreadyTaken")]
    #[cfg_attr(feature = "std", error("an org with the same ID already exists"))]
    DuplicateOrgId = 5,

    #[cfg_attr(feature = "std", error("a project with the same ID already exists"))]
    DuplicateProjectId = 6,

    #[deprecated(note = "Superseded by IdAlreadyTaken")]
    #[cfg_attr(feature = "std", error("a user with the same ID already exists."))]
    DuplicateUserId = 7,

    #[cfg_attr(feature = "std", error("the user is already a member of the org"))]
    AlreadyAMember = 8,

    #[cfg_attr(feature = "std", error("the provided fee is insufficient"))]
    InsufficientFee = 9,

    #[cfg_attr(feature = "std", error("the sender is not a project member"))]
    InsufficientSenderPermissions = 10,

    #[cfg_attr(
        feature = "std",
        error("the provided checkpoint is not a descendant of the project's initial checkpoint")
    )]
    InvalidCheckpointAncestry = 11,

    #[cfg_attr(
        feature = "std",
        error("the provided user is not eligible for unregistration")
    )]
    UnregisterableUser = 12,

    #[cfg_attr(
        feature = "std",
        error("the provided org is not elibile for unregistration")
    )]
    UnregisterableOrg = 13,

    #[cfg_attr(
        feature = "std",
        error("the account is already associated with a user")
    )]
    UserAccountAssociated = 14,

    #[cfg_attr(
        feature = "std",
        error("the tx author needs to have an associated user")
    )]
    AuthorHasNoAssociatedUser = 15,

    #[cfg_attr(
        feature = "std",
        error(
            "failed to update the chain runtime, ensure that the author is the chain's sudo key, \
            the 'spec_name' matches and the WASM 'spec_version' is greater"
        )
    )]
    FailedChainRuntimeUpdate = 16,

    #[cfg_attr(
        feature = "std",
        error("an org or a user with the same ID already exists")
    )]
    IdAlreadyTaken = 17,

    #[cfg_attr(
        feature = "std",
        error("the ID has been unregistered and can't be claimed again")
    )]
    IdRetired = 18,
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
