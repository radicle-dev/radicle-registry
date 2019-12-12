use alloc::prelude::v1::*;
use alloc::vec;
use codec::{Decode, Encode};
use frame_support::weights::SimpleDispatchInfo;
use frame_support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result as DispatchResult,
    storage::StorageMap as _,
    storage::StorageValue as _,
    traits::{Currency, ExistenceRequirement},
};

use sp_core::{crypto::UncheckedFrom, H256};

use frame_support::traits::Randomness;
use frame_system as system;
use frame_system::ensure_signed;

use crate::{AccountId, Balance, Hash, Hashing, String32};
use sp_runtime::traits::Hash as _;

/// The name a project is registered with.
pub type ProjectName = String32;

/// The domain under which the project's name is registered.
///
/// At present, the domain must be `rad`, alhtough others may be allowed in
/// the future.
pub type ProjectDomain = String32;

pub type ProjectId = (ProjectName, ProjectDomain);

pub type CheckpointId = H256;

/// A project's version. Used in checkpointing.
pub type Version = String;

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub id: ProjectId,
    pub account_id: AccountId,
    pub description: String,
    pub img_url: String,
    pub members: Vec<AccountId>,
    pub current_cp: CheckpointId,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterProjectParams {
    pub id: ProjectId,
    pub description: String,
    pub img_url: String,
    pub checkpoint_id: CheckpointId,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Checkpoint {
    pub parent: Option<CheckpointId>,
    pub hash: H256,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct CreateCheckpointParams {
    pub project_hash: H256,
    pub previous_checkpoint_id: Option<CheckpointId>,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct SetCheckpointParams {
    pub project_id: ProjectId,
    pub new_checkpoint_id: CheckpointId,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct TransferFromProjectParams {
    pub project: ProjectId,
    pub recipient: AccountId,
    pub value: Balance,
}

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

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn register_project(origin, params: RegisterProjectParams) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            if store::Checkpoints::get(params.checkpoint_id).is_none() {
                return Err("The checkpoint provided to register the project does not exist.")
            }
            match store::Projects::get(params.id.clone()) {
                None => {}
                Some (_) => return Err("A project with the supplied ID already exists."),
            };

            let project_id = params.id.clone();
            match store::Projects::get(project_id.clone()) {
                None => {}
                Some (_) => return Err("A project with the supplied ID already exists."),
            };
            let account_id = AccountId::unchecked_from(
                pallet_randomness_collective_flip::Module::<T>::random(b"project-account-id")
            );
            let project = Project {
                id: project_id.clone(),
                account_id: account_id,
                description: params.description,
                img_url: params.img_url,
                members: vec![sender],
                current_cp: params.checkpoint_id
            };

            store::Projects::insert(project_id.clone(), project);
            store::ProjectIds::append_or_put(vec![project_id.clone()]);
            store::InitialCheckpoints::insert(project_id.clone(), params.checkpoint_id);

            Self::deposit_event(Event::ProjectRegistered(project_id, account_id));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn transfer_from_project(origin, params: TransferFromProjectParams) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let project = store::Projects::get(params.project).ok_or("Project does not exist")?;
            let is_member = project.members.contains(&sender);
            if !is_member {
                return Err("Sender is not a project member")
            }
            <crate::Balances as Currency<_>>::transfer(&project.account_id, &params.recipient, params.value, ExistenceRequirement::KeepAlive)
        }

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn create_checkpoint(
            origin,
            params: CreateCheckpointParams,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            match params.previous_checkpoint_id {
                None => {}
                Some(cp_id) => {
                    match store::Checkpoints::get(cp_id) {
                        None => return Err("Parent checkpoint does not exist"),
                        Some(_) => {}
                    }
                }
            };

            let checkpoint = Checkpoint {
                parent: params.previous_checkpoint_id,
                hash: params.project_hash,
            };
            let checkpoint_id = Hashing::hash_of(&checkpoint);
            store::Checkpoints::insert(checkpoint_id, checkpoint);

            Self::deposit_event(Event::CheckpointCreated(checkpoint_id));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn set_checkpoint(
            origin,
            params: SetCheckpointParams,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            if store::Checkpoints::get(params.new_checkpoint_id).is_none() {
                return Err("The provided checkpoint does not exist")
            }
            let opt_project = store::Projects::get(params.project_id.clone());
            let new_project = match opt_project {
                None => return Err("The provided project ID is not associated with any project."),
                Some(prj) => {
                    if !prj.members.contains(&sender) {
                        return Err("The `set_checkpoint` transaction sender is not a member of the project.")
                    }
                    Project {
                        current_cp: params.new_checkpoint_id,
                        ..prj
                    }
                }
            };

            let initial_cp = match store::InitialCheckpoints::get(params.project_id.clone()) {
                None => return Err("A registered project must necessarily have an initial checkpoint."),
                Some(cp) => cp,
            };
            if !descends_from_initial_checkpoint(params.new_checkpoint_id, initial_cp) {
                return Err("The provided checkpoint ID is not a descendant of the project's initial checkpoint.")
            }

            store::Projects::insert(new_project.id.clone(), new_project.clone());

            Self::deposit_event(Event::CheckpointSet(new_project.id, params.new_checkpoint_id));
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
