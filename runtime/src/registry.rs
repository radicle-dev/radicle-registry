use alloc::prelude::v1::*;
use alloc::vec;
use codec::{Decode, Encode};
use sr_primitives::weights::SimpleDispatchInfo;
use srml_support::{
    decl_event, decl_module, decl_storage, dispatch::Result, storage::StorageMap as _,
    storage::StorageValue as _,
};
use srml_system as system;
use srml_system::ensure_signed;
use substrate_primitives::H256;

use crate::AccountId;

pub type ProjectId = H256;

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct Project {
    pub id: H256,
    pub description: String,
    pub name: String,
    pub img_url: String,
    pub members: Vec<AccountId>,
}

#[derive(Decode, Encode, Clone, Debug, Eq, PartialEq)]
pub struct RegisterProjectParams {
    pub id: H256,
    pub description: String,
    pub name: String,
    pub img_url: String,
}

pub trait Trait: srml_system::Trait<AccountId = AccountId, Origin = crate::Origin> {
    type Event: From<Event> + Into<<Self as srml_system::Trait>::Event>;
}

pub mod store {
    use super::*;

    decl_storage! {
        pub trait Store for Module<T: Trait> as Counter {
            pub Projects: map H256 => Option<Project>;
            pub ProjectIds: Vec<H256>;
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
            let project_id = params.id;
            let project = Project {
                id: project_id,
                name: params.name,
                description: params.description,
                img_url: params.img_url,
                members: vec![sender],
            };

            store::Projects::insert(project_id, project);
            store::ProjectIds::append_or_put(vec![project_id]);

            Self::deposit_event(Event::ProjectRegistered(project_id));
            Ok(())
        }
    }
}
decl_event!(
    pub enum Event {
        ProjectRegistered(H256),
    }
);
