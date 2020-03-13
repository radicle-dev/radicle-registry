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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Errors describing failed Registry transactions.
pub enum RegistryError {
    InexistentCheckpointId = 0,
    InexistentOrg,
    InexistentUser,
    DuplicateOrgId,
    DuplicateProjectId,
    DuplicateUserId,
    InexistentProjectId,
    InsufficientBid,
    InsufficientSenderPermissions,
    InexistentParentCheckpoint,
    InexistentInitialProjectCheckpoint,
    InvalidCheckpointAncestry,
    NonUnregisterableUser,
    UnregisterableOrg,
    UserAccountAssociated,
}

impl From<RegistryError> for &'static str {
    fn from(error: RegistryError) -> &'static str {
        match error {
            RegistryError::InexistentCheckpointId => "The provided checkpoint does not exist",
            RegistryError::InexistentOrg => "The provided org does not exist",
            RegistryError::InexistentUser => "The provided user does not exist",
            RegistryError::DuplicateOrgId => "An org with a similar ID already exists.",
            RegistryError::DuplicateProjectId => "A project with a similar ID already exists.",
            RegistryError::DuplicateUserId => "A user with the same ID already exists.",
            RegistryError::InexistentProjectId => "Project does not exist",
            RegistryError::InsufficientBid => {
                "The provided bid is insufficient to cover all mandatory fees."
            }
            RegistryError::InsufficientSenderPermissions => "Sender is not a project member",
            RegistryError::InexistentParentCheckpoint => "Parent checkpoint does not exist",
            RegistryError::InexistentInitialProjectCheckpoint => {
                "A registered project must have an initial checkpoint."
            }
            RegistryError::InvalidCheckpointAncestry => {
                "The provided checkpoint is not a descendant of the project's initial checkpoint."
            }
            RegistryError::NonUnregisterableUser => {
                "The provided user is not eligible for unregistration."
            }
            RegistryError::UnregisterableOrg => {
                "The provided org is not elibile for unregistration."
            }
            RegistryError::UserAccountAssociated => {
                "The account is already associated with a user."
            }
        }
    }
}

impl From<RegistryError> for DispatchError {
    fn from(error: RegistryError) -> Self {
        // This is the index with which the registry runtime module is declared
        // in the Radicle Registry runtime - see the `construct_runtime`
        // declaration in the `runtime` crate.
        let registry_index = 7;
        DispatchError::Module {
            index: registry_index,
            error: error as u8,
            message: None,
        }
    }
}
