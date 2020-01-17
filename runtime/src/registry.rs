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

use alloc::prelude::v1::*;
use alloc::vec;
use frame_support::weights::SimpleDispatchInfo;
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    storage::StorageMap as _,
    storage::StorageValue as _,
    traits::{Currency, ExistenceRequirement, Randomness as _},
};

use sp_core::crypto::UncheckedFrom;

use frame_system as system;
use frame_system::ensure_signed;

use crate::{AccountId, Hash, Hashing};
use sp_runtime::traits::Hash as _;

use radicle_registry_core::*;

pub trait Trait:
    frame_system::Trait<AccountId = AccountId, Origin = crate::Origin, Hash = Hash>
{
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;
}

pub mod store {
    use super::*;

    decl_storage! {
        pub trait Store for Module<T: Trait> as Counter {
            pub Projects: map ProjectId => Option<Project>;
            // The below map indexes each existing project's id to the
            // checkpoint id that it was registered with.
            pub InitialCheckpoints: map ProjectId => Option<CheckpointId>;
            pub ProjectIds: Vec<ProjectId>;
            // The below map indexes each checkpoint's id to the checkpoint
            // it points to, should it exist.
            pub Checkpoints: map CheckpointId => Option<Checkpoint>;
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
            let sender = ensure_signed(origin)?;

            if store::Checkpoints::get(message.checkpoint_id).is_none() {
                return Err(DispatchError::Other("The checkpoint provided to register the project does not exist."))
            }
            match store::Projects::get(message.id.clone()) {
                None => {}
                Some (_) => return Err(DispatchError::Other("A project with the supplied ID already exists.")),
            };

            let project_id = message.id.clone();
            match store::Projects::get(project_id.clone()) {
                None => {}
                Some (_) => return Err(DispatchError::Other("A project with the supplied ID already exists.")),
            };
            let account_id = AccountId::unchecked_from(
                pallet_randomness_collective_flip::Module::<T>::random(b"project-account-id")
            );
            let project = Project {
                id: project_id.clone(),
                account_id: account_id,
                members: vec![sender],
                current_cp: message.checkpoint_id
            };

            store::Projects::insert(project_id.clone(), project);
            store::ProjectIds::append_or_put(vec![project_id.clone()]);
            store::InitialCheckpoints::insert(project_id.clone(), message.checkpoint_id);

            Self::deposit_event(Event::ProjectRegistered(project_id, account_id));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn transfer_from_project(origin, message: message::TransferFromProject) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let project = store::Projects::get(message.project).ok_or("Project does not exist")?;
            let is_member = project.members.contains(&sender);
            if !is_member {
                return Err(DispatchError::Other("Sender is not a project member"))
            }
            <crate::Balances as Currency<_>>::transfer(&project.account_id, &message.recipient, message.value, ExistenceRequirement::KeepAlive)
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
                        None => return Err(DispatchError::Other("Parent checkpoint does not exist")),
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
            let sender = ensure_signed(origin)?;

            if store::Checkpoints::get(message.new_checkpoint_id).is_none() {
                return Err(DispatchError::Other("The provided checkpoint does not exist"))
            }
            let opt_project = store::Projects::get(message.project_id.clone());
            let new_project = match opt_project {
                None => return Err(DispatchError::Other("The provided project ID is not associated with any project.")),
                Some(prj) => {
                    if !prj.members.contains(&sender) {
                        return Err(DispatchError::Other("The `set_checkpoint` transaction sender is not a member of the project."))
                    }
                    Project {
                        current_cp: message.new_checkpoint_id,
                        ..prj
                    }
                }
            };

            let initial_cp = match store::InitialCheckpoints::get(message.project_id.clone()) {
                None => return Err(DispatchError::Other("A registered project must necessarily have an initial checkpoint.")),
                Some(cp) => cp,
            };
            if !descends_from_initial_checkpoint(message.new_checkpoint_id, initial_cp) {
                return Err(DispatchError::Other("The provided checkpoint ID is not a descendant of the project's initial checkpoint."))
            }

            store::Projects::insert(new_project.id.clone(), new_project.clone());

            Self::deposit_event(Event::CheckpointSet(new_project.id, message.new_checkpoint_id));
            Ok(())
        }
    }
}
decl_event!(
    pub enum Event {
        ProjectRegistered(ProjectId, AccountId),
        CheckpointCreated(CheckpointId),
        CheckpointSet(ProjectId, CheckpointId),
    }
);
