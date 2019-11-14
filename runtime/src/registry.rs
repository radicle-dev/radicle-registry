use alloc::prelude::v1::*;
use alloc::vec;
use codec::{Decode, Encode};
use sr_primitives::weights::SimpleDispatchInfo;
use srml_support::{
    decl_event, decl_module, decl_storage, dispatch::Result, storage::StorageMap as _,
    storage::StorageValue as _,
};

use substrate_primitives::H256;

use srml_system as system;
use srml_system::ensure_signed;

use crate::AccountId;

/// The name a project is registered with.
pub type ProjectName = String;

/// The domain under which the project's name is registered.
///
/// At present, the domain must be `rad`, alhtough others may be allowed in
/// the future.
pub type ProjectDomain = String;

pub type ProjectId = (ProjectName, ProjectDomain);

pub type CheckpointId = H256;

/// A project's version. Used in checkpointing.
pub type Version = String;

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub id: ProjectId,
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
    pub checkpoint_id: CheckpointId,
    pub project_hash: H256,
    pub previous_checkpoint: Option<CheckpointId>,
}

pub trait Trait: srml_system::Trait<AccountId = AccountId, Origin = crate::Origin> {
    type Event: From<Event> + Into<<Self as srml_system::Trait>::Event>;
}

pub mod store {
    use super::*;

    decl_storage! {
        pub trait Store for Module<T: Trait> as Counter {
            pub Projects: map ProjectId => Option<Project>;
            pub ProjectIds: Vec<ProjectId>;
            pub Checkpoints: map CheckpointId => Option<Checkpoint>;
        }
    }
}

pub use store::Store;

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        fn deposit_event() = default;

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn register_project(origin, params: RegisterProjectParams) -> Result {
            let sender = ensure_signed(origin)?;
            let project_id = params.id.clone();
            let project = Project {
                id: project_id.clone(),
                description: params.description,
                img_url: params.img_url,
                members: vec![sender],
                current_cp: params.checkpoint_id
            };

            store::Projects::insert(project_id.clone(), project);
            store::ProjectIds::append_or_put(vec![project_id.clone()]);

            Self::deposit_event(Event::ProjectRegistered(project_id.clone()));
            Ok(())
        }

        #[weight = SimpleDispatchInfo::FreeNormal]
        pub fn create_checkpoint(
            origin,
            params: CreateCheckpointParams,
        ) -> Result {
            ensure_signed(origin)?;
            let checkpoint = Checkpoint {
                parent: params.previous_checkpoint,
                hash: params.project_hash,
            };
            store::Checkpoints::insert(params.checkpoint_id, checkpoint);

            Self::deposit_event(Event::CheckpointCreated(params.checkpoint_id));
            Ok(())
        }
    }
}
decl_event!(
    pub enum Event {
        ProjectRegistered(ProjectId),
        CheckpointCreated(CheckpointId),
    }
);
