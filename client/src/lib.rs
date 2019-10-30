//! Client for the radicle registry
//!
//! The client comes in two flavours: The [Client] with methods that return [Future]s and the
//! [SyncClient] with the same methods, but returning a corresponding [Result] instead of a
//! [Future]. The latter is usefull for writing synchronous code and spawns work in a separate
//! runtime.
use futures01::prelude::*;

use radicle_registry_runtime::{balances, counter, registry, Address};
use substrate_subxt::balances::BalancesStore;

mod base;
mod sync;

pub use substrate_primitives::crypto::Pair;
pub use substrate_primitives::ed25519;

#[doc(inline)]
pub use radicle_registry_runtime::{
    counter::CounterValue,
    registry::{Project, ProjectId},
    Balance,
};

#[doc(inline)]
pub use base::Error;
pub use sync::SyncClient;

/// Account IDs are Ed25519 public keys
pub type AccountId = ed25519::Public;

#[derive(Clone, Debug)]
pub struct RegisterProjectParams {
    pub description: String,
    pub name: String,
    pub img_url: String,
}

/// Client to interact with the radicle registry ledger through a local node.
pub struct Client {
    base_client: base::Client,
}

impl Client {
    /// Connects to a registry node running on localhost and returns a [Client].
    ///
    /// Fails if it cannot connect to a node.
    pub fn create() -> impl Future<Item = Self, Error = Error> {
        base::Client::create().map(|base_client| Client { base_client })
    }

    pub fn transfer(
        &self,
        key_pair: &ed25519::Pair,
        receiver: &AccountId,
        balance: Balance,
    ) -> impl Future<Item = (), Error = Error> {
        self.base_client
            .submit_and_watch_call(
                key_pair,
                balances::Call::transfer(Address::Id(receiver.clone()), balance),
            )
            .map(|_| ())
    }

    pub fn free_balance(
        &self,
        account_id: &AccountId,
    ) -> impl Future<Item = Balance, Error = Error> {
        self.base_client
            .subxt_client
            .free_balance(account_id.clone())
    }

    pub fn counter_inc(&self, key_pair: &ed25519::Pair) -> impl Future<Item = (), Error = Error> {
        self.base_client
            .submit_and_watch_call(key_pair, counter::Call::inc())
            .map(|_| ())
    }

    pub fn get_counter(&self) -> impl Future<Item = Option<CounterValue>, Error = Error> {
        self.base_client.fetch_value::<counter::Value, _>()
    }

    pub fn register_project(
        &self,
        author: &ed25519::Pair,
        project_params: RegisterProjectParams,
    ) -> impl Future<Item = ProjectId, Error = Error> {
        let id = ProjectId::random();
        self.base_client
            .submit_and_watch_call(
                author,
                registry::Call::register_project(registry::RegisterProjectParams {
                    id,
                    name: project_params.name,
                    description: project_params.description,
                    img_url: project_params.img_url,
                }),
            )
            .map(move |_| id)
    }

    pub fn get_project(&self, id: ProjectId) -> impl Future<Item = Option<Project>, Error = Error> {
        self.base_client
            .fetch_map_value::<registry::store::Projects, _, _>(id)
    }

    pub fn list_projects(&self) -> impl Future<Item = Vec<ProjectId>, Error = Error> {
        self.base_client
            .fetch_value::<registry::store::ProjectIds, _>()
            .map(|maybe_ids| maybe_ids.unwrap_or(Vec::new()))
    }
}
