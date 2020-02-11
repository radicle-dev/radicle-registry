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

use alloc::vec;
use alloc::vec::Vec;

use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    storage::StorageMap as _,
    traits::{Currency, ExistenceRequirement, Randomness as _},
    weights::SimpleDispatchInfo,
};
use frame_system as system; // required for `decl_module!` to work
use frame_system::ensure_signed;
use sp_core::crypto::UncheckedFrom;
use sp_runtime::traits::Hash as _;

use radicle_registry_core::*;

use crate::{AccountId, Hash, Hashing};

pub trait Trait:
    frame_system::Trait<AccountId = AccountId, Origin = crate::Origin, Hash = Hash>
{
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
}

pub mod store {
    use super::*;

    decl_storage! {
        pub trait Store for Module<T: Trait> as Counter {
            // The storage for Orgs, indexed by OrgId.
            // We use the blake2_128_concat hasher so that the OrgId
            // can be extracted from the key.
            pub Orgs: map hasher(blake2_128_concat) OrgId => Option<state::Org>;

            // We use the blake2_128_concat hasher so that the ProjectId can be extracted from the
            // key.
            pub Projects: map hasher(blake2_128_concat) ProjectId => Option<state::Project>;
            // The below map indexes each existing project's id to the
            // checkpoint id that it was registered with.
            pub InitialCheckpoints: map hasher(blake2_256) ProjectId => Option<CheckpointId>;
            // The below map indexes each checkpoint's id to the checkpoint
            // it points to, should it exist.
            pub Checkpoints: map hasher(blake2_256) CheckpointId => Option<state::Checkpoint>;
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
        ///     OrgId::from_str("Monadic").unwrap(),
        ///     ProjectName::from_str("radicle").unwrap()
        /// );
        ///
        /// let key = store::Projects::storage_map_final_key(project_id.clone());
        /// let extracted_project_id = store::Projects::id_from_key(&key).unwrap();
        /// assert_eq!(project_id, extracted_project_id)
        /// ```
        pub fn id_from_key(key: &[u8]) -> Result<ProjectId, parity_scale_codec::Error> {
            use parity_scale_codec::Decode;

            let project_prefix = Self::final_prefix();
            // Length of BlakeTwo128 output
            let key_hash_prefix_length = 16;
            let key_prefix_length = project_prefix.len() + key_hash_prefix_length;
            let mut project_id_bytes = &key[key_prefix_length..];
            ProjectId::decode(&mut project_id_bytes)
        }
    }

    #[cfg(feature = "std")]
    impl Orgs {
        /// Get the org ID from the orgs storage key.
        ///
        /// The following property holds.
        /// ```
        /// # use radicle_registry_core::*;
        /// # use radicle_registry_runtime::registry::store;
        /// # use frame_support::storage::generator::StorageMap;
        /// # use std::str::FromStr;
        /// let org_id = OrgId::from_str("org").unwrap();
        /// let key = store::Orgs::storage_map_final_key(org_id.clone());
        /// let extracted_org_id = store::Orgs::id_from_key(&key).unwrap();
        /// assert_eq!(org_id, extracted_org_id)
        /// ```
        pub fn id_from_key(key: &[u8]) -> Result<OrgId, parity_scale_codec::Error> {
            use parity_scale_codec::Decode;

            let prefix = Self::final_prefix();
            // Length of BlakeTwo128 output
            let key_hash_prefix_length = 16;
            let key_prefix_length = prefix.len() + key_hash_prefix_length;
            let mut id_bytes = &key[key_prefix_length..];
            OrgId::decode(&mut id_bytes)
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

        #[weight = SimpleDispatchInfo::InsecureFreeNormal]
        pub fn register_project(origin, message: message::RegisterProject) -> DispatchResult {
            let _sender = ensure_signed(origin)?;

            if store::Checkpoints::get(message.checkpoint_id).is_none() {
                return Err(RegistryError::InexistentCheckpointId.into())
            }

            let (org_id, project_name) = message.id.clone();

            if store::Orgs::get(org_id.clone()).is_none() {
                return Err(RegistryError::InexistentOrg.into());
            }

            let project_id = message.id.clone();
            if store::Projects::get(project_id.clone()).is_some() {
                return Err(RegistryError::DuplicateProjectId.into());
            };

            let new_project = state::Project {
                current_cp: message.checkpoint_id,
                metadata: message.metadata
            };

            store::Projects::insert(project_id.clone(), new_project);
            store::Orgs::insert(
                org_id.clone(),
                store::Orgs::get(org_id).unwrap().add_project(project_name)
            );
            store::InitialCheckpoints::insert(project_id.clone(), message.checkpoint_id);

            Self::deposit_event(Event::ProjectRegistered(project_id));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::InsecureFreeNormal]
        pub fn register_org(origin, message: message::RegisterOrg) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            match store::Orgs::get(message.id.clone()) {
                None => {},
                Some(_) => return Err(RegistryError::DuplicateOrgId.into()),
            }

            let random_account_id = AccountId::unchecked_from(
                pallet_randomness_collective_flip::Module::<T>::random(
                    b"org-account-id",
                )
            );

            let new_org = state::Org {
                account_id: random_account_id,
                members: vec![sender],
                projects: Vec::new(),
            };
            store::Orgs::insert(message.id.clone(), new_org);
            Self::deposit_event(Event::OrgRegistered(message.id));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::InsecureFreeNormal]
        pub fn unregister_org(origin, message: message::UnregisterOrg) -> DispatchResult {
            fn can_be_unregistered(org: state::Org, sender: AccountId) -> bool {
                org.members == vec![sender] && org.projects.is_empty()
            }

            let sender = ensure_signed(origin)?;

            match store::Orgs::get(message.id.clone()) {
                None => Err(RegistryError::InexistentOrg.into()),
                Some(org) => {
                    if can_be_unregistered(org, sender) {
                        store::Orgs::remove(message.id.clone());
                        Self::deposit_event(Event::OrgUnregistered(message.id));
                        Ok(())
                    }
                    else {
                        Err(RegistryError::UnregisterableOrg.into())
                    }
                }
            }
        }

        #[weight = SimpleDispatchInfo::InsecureFreeNormal]
        pub fn transfer_from_org(origin, message: message::TransferFromOrg) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let org = match store::Orgs::get(message.org_id) {
                None => return Err(RegistryError::InexistentOrg.into()),
                Some(o) => o,
            };
            if org.members.contains(&sender) {
                <crate::Balances as Currency<_>>::transfer(
                    &org.account_id,
                    &message.recipient,
                    message.value, ExistenceRequirement::KeepAlive
                )
            }
            else {
                Err(RegistryError::InsufficientSenderPermissions.into())
            }
        }

        #[weight = SimpleDispatchInfo::InsecureFreeNormal]
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

            let checkpoint = state::Checkpoint {
                parent: message.previous_checkpoint_id,
                hash: message.project_hash,
            };
            let checkpoint_id = Hashing::hash_of(&checkpoint);
            store::Checkpoints::insert(checkpoint_id, checkpoint);

            Self::deposit_event(Event::CheckpointCreated(checkpoint_id));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::InsecureFreeNormal]
        pub fn set_checkpoint(
            origin,
            message: message::SetCheckpoint,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            if store::Checkpoints::get(message.new_checkpoint_id).is_none() {
                return Err(RegistryError::InexistentCheckpointId.into())
            }
            let project_id = message.project_id.clone();
            let opt_project = store::Projects::get(project_id.clone());
            let opt_org = store::Orgs::get(project_id.0.clone());
            let new_project = match (opt_project, opt_org) {
                (Some(prj), Some(org)) => {
                    if !org.members.contains(&sender) {
                        return Err(RegistryError::InsufficientSenderPermissions.into())
                    }
                    state::Project {
                        current_cp: message.new_checkpoint_id,
                        ..prj
                    }
                }
                _ => return Err(RegistryError::InexistentProjectId.into()),

            };

            let initial_cp = match store::InitialCheckpoints::get(project_id.clone()) {
                None => return Err(RegistryError::InexistentInitialProjectCheckpoint.into()),
                Some(cp) => cp,
            };
            if !descends_from_initial_checkpoint(message.new_checkpoint_id, initial_cp) {
                return Err(RegistryError::InvalidCheckpointAncestry.into())
            }

            store::Projects::insert(project_id.clone(), new_project);

            Self::deposit_event(Event::CheckpointSet(project_id, message.new_checkpoint_id));
            Ok(())
        }
    }
}
decl_event!(
    pub enum Event {
        OrgUnregistered(OrgId),
        OrgRegistered(OrgId),
        ProjectRegistered(ProjectId),
        CheckpointCreated(CheckpointId),
        CheckpointSet(ProjectId, CheckpointId),
    }
);
