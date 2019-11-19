//! Clients for the radicle registry.
//!
//! This crate provides two high-level clients for the radicle registry. [Client] talks to a
//! radicle registry node. [MemoryClient] provides the same interface but runs the ledger in
//! memory. This is useful for developing and testing.
use futures01::prelude::*;

use radicle_registry_runtime::{balances, registry, Runtime};

mod base;
mod memory;
mod with_executor;

pub use radicle_registry_client_interface::{Client as ClientT, *};

pub use crate::base::Error;
pub use crate::memory::MemoryClient;
pub use crate::with_executor::ClientWithExecutor;

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
}

impl ClientT for Client {
    fn submit<Call_: Call>(
        &self,
        author: &ed25519::Pair,
        call: Call_,
    ) -> Response<TransactionApplied<Call_>, Error> {
        let base_client = self.base_client.clone();
        Box::new(
            self.base_client
                .submit_runtime_call(author, call.into_runtime_call())
                .and_then(move |ext_success| {
                    let tx_hash = ext_success.extrinsic;
                    let block = ext_success.block;
                    base_client
                        .extract_events(ext_success)
                        .and_then(move |events| {
                            let result = Call_::result_from_events(events.clone())?;
                            Ok(TransactionApplied {
                                tx_hash,
                                block,
                                events,
                                result,
                            })
                        })
                }),
        )
    }

    fn get_transaction_extra(&self, account_id: &AccountId) -> Response<TransactionExtra, Error> {
        self.base_client.get_transaction_extra(account_id)
    }

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Error> {
        Box::new(
            self.base_client
                .fetch_map_value::<balances::FreeBalance<Runtime>, _, _>(account_id.clone()),
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
                .fetch_value::<registry::store::ProjectIds, _>(),
        )
    }

    fn get_checkpoint(&self, id: CheckpointId) -> Response<Option<Checkpoint>, Error> {
        Box::new(
            self.base_client
                .fetch_map_value::<registry::store::Checkpoints, _, _>(id),
        )
    }
}
