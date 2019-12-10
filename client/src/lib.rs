//! Clients for the radicle registry.
//!
//! This crate provides a high-level registry ledger [Client] and all related types.
//!
//! Create a remote node client with [Client::create].
//!
//! [Client::new_emulator] creates a client that emulates the ledger in memory without having a
//! local node.
//!
//! [Client::create_with_executor] creates a client that uses its own runtime to spawn futures.
use futures01::prelude::*;
use std::sync::Arc;

use parity_scale_codec::{Decode, FullCodec};

use paint_support::storage::generator::{StorageMap, StorageValue};
use radicle_registry_runtime::{balances, registry, Runtime};

mod backend;
mod call;
mod extrinsic;
mod interface;

pub use crate::call::Call;
pub use crate::interface::{Client as ClientT, *};

/// Client to interact with the radicle registry ledger via an implementation of [ClientT].
///
/// The client can either use a full node as the backend (see [Client::create]) or emulate the
/// registry in memory with [Client::new_emulator].
pub struct Client {
    backend: Arc<dyn backend::Backend + Sync + Send>,
}

impl Client {
    /// Connects to a registry node running on localhost and returns a [Client].
    ///
    /// Fails if it cannot connect to a node.
    pub fn create() -> impl Future<Item = Self, Error = Error> {
        backend::RemoteNode::create().map(Self::new)
    }

    /// Same as [Client::create] but calls to the client spawn futures in an executor owned by the
    /// client.
    ///
    /// This makes it possible to call [Future::wait] on the client even if that function is called
    /// in an event loop of another executor.
    pub fn create_with_executor() -> Result<Self, Error> {
        backend::RemoteNodeWithExecutor::create().map(Self::new)
    }

    /// Create a new client that emulates the registry ledger in memory. See
    /// [backend::emulator::Emulator] for details.
    pub fn new_emulator() -> Self {
        Self::new(backend::Emulator::new())
    }

    fn new(backend: impl backend::Backend + Sync + Send + 'static) -> Self {
        Client {
            backend: Arc::new(backend),
        }
    }

    /// Fetch a value from the state storage based on a [StorageValue] implementation provided by
    /// the runtime.
    ///
    /// ```ignore
    /// client.fetch_value::<paint_balance::TotalIssuance<Runtime>, _>();
    /// ```
    fn fetch_value<S: StorageValue<Value>, Value: FullCodec + Send + 'static>(
        &self,
    ) -> Response<S::Query, Error>
    where
        S::Query: Send + 'static,
    {
        Box::new(
            self.backend
                .fetch(S::storage_value_final_key().as_ref())
                .and_then(|maybe_data| {
                    let value = match maybe_data {
                        Some(data) => {
                            let value = Decode::decode(&mut &data[..])?;
                            Some(value)
                        }
                        None => None,
                    };
                    Ok(S::from_optional_value_to_query(value))
                }),
        )
    }

    /// Fetch a value from a map in the state storage based on a [StorageMap] implementation
    /// provided by the runtime.
    ///
    /// ```ignore
    /// client.fetch_map_value::<paint_system::AccountNonce<Runtime>, _, _>(account_id);
    /// ```
    fn fetch_map_value<
        S: StorageMap<Key, Value>,
        Key: FullCodec,
        Value: FullCodec + Send + 'static,
    >(
        &self,
        key: Key,
    ) -> Response<S::Query, Error>
    where
        S::Query: Send + 'static,
    {
        Box::new(
            self.backend
                .fetch(S::storage_map_final_key(key).as_ref())
                .and_then(|maybe_data| {
                    let value = match maybe_data {
                        Some(data) => {
                            let value = Decode::decode(&mut &data[..])?;
                            Some(value)
                        }
                        None => None,
                    };
                    Ok(S::from_optional_value_to_query(value))
                }),
        )
    }
}

impl ClientT for Client {
    fn submit<Call_: Call>(
        &self,
        author: &ed25519::Pair,
        call: Call_,
    ) -> Response<TransactionApplied<Call_>, Error> {
        let account_id = author.public();
        let key_pair = author.clone();
        let backend2 = self.backend.clone();
        Box::new(
            self.backend
                .get_transaction_extra(&account_id)
                .and_then(move |extra| {
                    let extrinsic = crate::extrinsic::signed_extrinsic(
                        &key_pair,
                        call.into_runtime_call(),
                        extra.nonce,
                        extra.genesis_hash,
                    );
                    backend2.submit(extrinsic).and_then(|tx_applied| {
                        let events = tx_applied.events;
                        let tx_hash = tx_applied.tx_hash;
                        let block = tx_applied.block;
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

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Error> {
        Box::new(self.fetch_map_value::<balances::FreeBalance<Runtime>, _, _>(account_id.clone()))
    }

    fn get_project(&self, id: ProjectId) -> Response<Option<Project>, Error> {
        Box::new(self.fetch_map_value::<registry::store::Projects, _, _>(id))
    }

    fn list_projects(&self) -> Response<Vec<ProjectId>, Error> {
        Box::new(self.fetch_value::<registry::store::ProjectIds, _>())
    }

    fn get_checkpoint(&self, id: CheckpointId) -> Response<Option<Checkpoint>, Error> {
        Box::new(self.fetch_map_value::<registry::store::Checkpoints, _, _>(id))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Assert that [Client] implements [Sync], [Send] and has a `'static` lifetime bound.
    ///
    /// The code does not need to run, we only want it to compile.
    #[allow(dead_code)]
    fn client_is_sync_send_static() {
        fn is_sync_send(_x: impl Sync + Send + 'static) {}
        is_sync_send(Client::new_emulator());
    }
}
