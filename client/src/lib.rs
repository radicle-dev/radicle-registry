//! Node client for the radicle registry
use futures01::prelude::*;

use radicle_registry_runtime::{counter, registry};
use substrate_subxt::balances::BalancesStore as _;

mod base;
mod with_executor;

pub use radicle_registry_client_interface::{Client as ClientT, *};
pub use radicle_registry_runtime::counter::CounterValue;

pub use base::Error;
pub use with_executor::ClientWithExecutor;

/// Client to interact with the radicle registry ledger through a local node.
///
/// Implements [ClientT] for interacting with the ledger.
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

    pub fn counter_inc(&self, key_pair: &ed25519::Pair) -> impl Future<Item = (), Error = Error> {
        self.base_client
            .submit_runtime_call(key_pair, counter::Call::inc().into())
            .map(|_| ())
    }

    pub fn get_counter(&self) -> impl Future<Item = Option<CounterValue>, Error = Error> {
        self.base_client.fetch_value::<counter::Value, _>()
    }
}

impl ClientT for Client {
    fn submit(&self, author: &ed25519::Pair, call: Call) -> Response<TxHash, Error> {
        Box::new(
            self.base_client
                .submit_runtime_call(
                    author,
                    radicle_registry_client_common::into_runtime_call(call),
                )
                .map(|xt| xt.extrinsic),
        )
    }

    fn get_transaction_extra(&self, account_id: &AccountId) -> Response<TransactionExtra, Error> {
        self.base_client.get_transaction_extra(account_id)
    }

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Error> {
        Box::new(
            self.base_client
                .subxt_client
                .free_balance(account_id.clone()),
        )
    }
    fn get_project(&self, id: ProjectId) -> Response<Option<Project>, Error> {
        Box::new(
            self.base_client
                .fetch_map_value::<registry::store::Projects, _, _>(id),
        )
    }

    fn list_projects(&self) -> Response<Vec<ProjectId>, Error> {
        Box::new(
            self.base_client
                .fetch_value::<registry::store::ProjectIds, _>()
                .map(|maybe_ids| maybe_ids.unwrap_or_default()),
        )
    }

    fn get_checkpoint(&self, id: CheckpointId) -> Response<Option<Checkpoint>, Error> {
        Box::new(
            self.base_client
                .fetch_map_value::<registry::store::Checkpoints, _, _>(id),
        )
    }
}
