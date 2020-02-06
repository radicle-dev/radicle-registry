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

use frame_support::{
    decl_event, decl_module, decl_storage, dispatch::DispatchResult, storage::StorageMap as _,
    weights::SimpleDispatchInfo,
};
use frame_system as system; // required for `decl_module!` to work
use frame_system::ensure_signed;
use sp_runtime::traits::Hash as _;

use radicle_registry_core::*;

use crate::{AccountId, Hash, Hashing};
use core::str::FromStr;

type GlobalProjectId = (ProjectId, OrgId);

pub trait Trait:
    frame_system::Trait<AccountId = AccountId, Origin = crate::Origin, Hash = Hash>
{
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
}

pub mod store {
    use super::*;

    decl_storage! {
        pub trait Store for Module<T: Trait> as Counter {
            // We use the blake2_128_concat hasher so that the GlobalProjectId can be extracted from the
            // key.
            pub Projects: map hasher(blake2_128_concat) GlobalProjectId => Option<Project>;
            // The below map indexes each existing project's id to the
            // checkpoint id that it was registered with.
            pub InitialCheckpoints: map ProjectId => Option<CheckpointId>;
            // The below map indexes each checkpoint's id to the checkpoint
            // it points to, should it exist.
            pub Checkpoints: map CheckpointId => Option<Checkpoint>;
        }
    }

    #[cfg(feature = "std")]
    impl Projects {
        /// Get the project ID from the projects storage key.
        ///
        /// The following property holds.
        /// ```
        /// # use radicle_registry_core::*;
        /// # use radicle_registry_runtime::registry::store;
        /// # use frame_support::storage::generator::StorageMap;
        /// # use std::str::FromStr;
        /// let project_id = (
        ///     String32::from_str("name").unwrap(),
        ///     ProjectDomain::from_str("rad").unwrap()
        /// );
        ///
        /// let key = store::Projects::storage_map_final_key(project_id.clone());
        /// let extracted_project_id = store::Projects::id_from_key(&key).unwrap();
        /// assert_eq!(project_id, extracted_project_id)
        /// ```
        pub fn id_from_key(key: &[u8]) -> Result<GlobalProjectId, parity_scale_codec::Error> {
            use parity_scale_codec::Decode;

            let project_prefix = Self::final_prefix();
            // Length of BlakeTwo128 output
            let key_hash_prefix_length = 16;
            let key_prefix_length = project_prefix.len() + key_hash_prefix_length;
            let mut project_id_bytes = &key[key_prefix_length..];
            GlobalProjectId::decode(&mut project_id_bytes)
        }
    }
}

pub use store::Store;

/// Returns true iff `checkpoint_id` descends from `initial_cp_id`.
fn descends_from_initial_checkpoint(
    checkpoint_id: CheckpointId,
    initial_cp_id: CheckpointId,
) -> bool {
    if checkpoint_id == initial_cp_id {
        return true;
    };

    let mut ancestor_id = checkpoint_id;

    // The number of storage requests made in this loop grows linearly
    // with the size of the checkpoint's ancestry.
    //
    // The loop's total runtime will also depend on the performance of
    // each `store::StorageMap::get` request.
    while let Some(cp) = store::Checkpoints::get(ancestor_id) {
        match cp.parent {
            None => return false,
            Some(cp_id) => {
                if cp_id == initial_cp_id {
                    return true;
                } else {
                    ancestor_id = cp_id;
                }
            }
        }
    }

    false
}

// TODO Note on `DispatchError`
//
// This datatype is now an `enum`, and some of its variants can be used to
// provide richer errors in the case of runtime failures.
//
// This will be handled in a future issue.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn register_project(origin, message: message::RegisterProject) -> DispatchResult {
            let _sender: AccountId = ensure_signed(origin)?;

            if store::Checkpoints::get(message.checkpoint_id).is_none() {
                return Err(RegistryError::InexistentCheckpointId.into())
            }

            let project_id = message.id.clone();
            let global_project_id = (project_id.clone(), message.org_id.clone());

            match store::Projects::get(global_project_id.clone()) {
                None => {}
                Some (_) => return Err(RegistryError::DuplicateProjectId.into()),
            };

            let project = Project {
                id: project_id.clone(),
                org_id: message.org_id.clone(),
                current_cp: message.checkpoint_id,
                metadata: message.metadata
            };

            store::Projects::insert(global_project_id, project);
            store::InitialCheckpoints::insert(project_id.clone(), message.checkpoint_id);

            Self::deposit_event(Event::ProjectRegistered(project_id, message.org_id));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::FreeNormal]
        //TODO(nuno): delete this
        pub fn transfer_from_project(origin, message: message::TransferFromProject) -> DispatchResult {
            let _sender = ensure_signed(origin)?;
            let tmp_org_id = OrgId::from_str("tmp_org").unwrap();
            let global_project_id = (message.project, tmp_org_id);
            let _project = match store::Projects::get(global_project_id) {
                None => return Err(RegistryError::InexistentProjectId.into()),
                Some(p) => p,
            };
            // let is_member = project.members.contains(&sender);
            // if !is_member {Ø
                Err(RegistryError::InsufficientSenderPermissions.into())
            // }
            // <crate::Balances as Currency<_>>::transfer(
            //     &project.account_id,
            //     &message.recipient,
            //     message.value,
            //     ExistenceRequirement::KeepAlive
            // )
        }

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn create_checkpoint(
            origin,
            message: message::CreateCheckpoint,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            match message.previous_checkpoint_id {
                None => {}
                Some(cp_id) => {
                    match store::Checkpoints::get(cp_id) {
                        None => return Err(RegistryError::InexistentCheckpointId.into()),
                        Some(_) => {}
                    }
                }
            };

            let checkpoint = Checkpoint {
                parent: message.previous_checkpoint_id,
                hash: message.project_hash,
            };
            let checkpoint_id = Hashing::hash_of(&checkpoint);
            store::Checkpoints::insert(checkpoint_id, checkpoint);

            Self::deposit_event(Event::CheckpointCreated(checkpoint_id));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn set_checkpoint(
            origin,
            message: message::SetCheckpoint,
        ) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            if store::Checkpoints::get(message.new_checkpoint_id).is_none() {
                return Err(RegistryError::InexistentCheckpointId.into())
            }
            let global_project_id = (message.project_id.clone(), message.project_org_id.clone());
            let opt_project = store::Projects::get(global_project_id.clone());
            let new_project = match opt_project {
                None => return Err(RegistryError::InexistentProjectId.into()),
                Some(prj) => {
                    //TODO(nuno): Check whether the sender is part of the project's org.
                    if false { //if !prj.members.contains(&sender) {
                        return Err(RegistryError::InsufficientSenderPermissions.into())
                    }
                    Project {
                        current_cp: message.new_checkpoint_id,
                        ..prj
                    }
                }
            };

            let initial_cp = match store::InitialCheckpoints::get(message.project_id.clone()) {
                None => return Err(RegistryError::InexistentInitialProjectCheckpoint.into()),
                Some(cp) => cp,
            };
            if !descends_from_initial_checkpoint(message.new_checkpoint_id, initial_cp) {
                return Err(RegistryError::InvalidCheckpointAncestry.into())
            }

            store::Projects::insert(global_project_id, new_project.clone());

            Self::deposit_event(Event::CheckpointSet(new_project.id, message.new_checkpoint_id));
            Ok(())
        }
    }
}
decl_event!(
    pub enum Event {
        ProjectRegistered(ProjectId, OrgId),
        CheckpointCreated(CheckpointId),
        CheckpointSet(ProjectId, CheckpointId),
    }
);
