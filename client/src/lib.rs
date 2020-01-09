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
//!
//! # Transactions
//!
//! A [Transaction] can be created and signed offline using [Transaction::new_signed]. This
//! constructor requires the account nonce and genesis hash of the chain. Those can be obtained
//! using [ClientT::account_nonce] and [ClientT::genesis_hash]. See
//! `./client/examples/transaction_signing.rs`.
//!
//! # Async executor
//!
//! If the client is created with [Client::create] the futures returned by the client must be
//! spawned in a `tokio` v0.1.22 executor.
//!
use futures01::prelude::*;
use futures03::compat::Future01CompatExt;
use futures03::future::Future as Future03;
use std::sync::Arc;

use parity_scale_codec::{Decode, FullCodec};

use frame_support::storage::generator::{StorageMap, StorageValue};
use radicle_registry_runtime::{balances, registry, Runtime};

mod backend;
mod call;
mod error;
mod interface;
mod transaction;

pub use crate::interface::*;

/// Client to interact with the radicle registry ledger via an implementation of [ClientT].
///
/// The client can either use a full node as the backend (see [Client::create]) or emulate the
/// registry in memory with [Client::new_emulator].
#[derive(Clone)]
pub struct Client {
    backend: Arc<dyn backend::Backend + Sync + Send>,
}

impl Client {
    /// Connects to a registry node running on localhost and returns a [Client].
    ///
    /// Fails if it cannot connect to a node.
    pub fn create() -> impl Future<Item = Self, Error = Error> {
        future03_compat(backend::RemoteNode::create()).map(Self::new)
    }

    /// Same as [Client::create] but calls to the client spawn futures in an executor owned by the
    /// client.
    ///
    /// This makes it possible to call [Future::wait] on the client even if that function is called
    /// in an event loop of another executor.
    pub fn create_with_executor() -> Result<Self, Error> {
        future03_compat(backend::RemoteNodeWithExecutor::create())
            .map(Self::new)
            .wait()
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
    /// client.fetch_value::<frame_balance::TotalIssuance<Runtime>, _>();
    /// ```
    fn fetch_value<S: StorageValue<Value>, Value: FullCodec + Send + 'static>(
        &self,
    ) -> Response<S::Query, Error>
    where
        S::Query: Send + 'static,
    {
        let backend = self.backend.clone();
        future03_compat(async move {
            let maybe_data = backend
                .fetch(S::storage_value_final_key().as_ref(), None)
                .await?;
            let value = match maybe_data {
                Some(data) => {
                    let value = Decode::decode(&mut &data[..])?;
                    Some(value)
                }
                None => None,
            };
            Ok(S::from_optional_value_to_query(value))
        })
    }

    /// Fetch a value from a map in the state storage based on a [StorageMap] implementation
    /// provided by the runtime.
    ///
    /// ```ignore
    /// client.fetch_map_value::<frame_system::AccountNonce<Runtime>, _, _>(account_id);
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
        let backend = self.backend.clone();
        // We cannot move this code into the async block. The compiler complains about a processing
        // cycle (E0391)
        let key = S::storage_map_final_key(key);
        future03_compat(async move {
            let maybe_data = backend.fetch(&key, None).await?;
            let value = match maybe_data {
                Some(data) => {
                    let value = Decode::decode(&mut &data[..])?;
                    Some(value)
                }
                None => None,
            };
            Ok(S::from_optional_value_to_query(value))
        })
    }
}

impl ClientT for Client {
    fn submit_transaction<Call_: Call>(
        &self,
        transaction: Transaction<Call_>,
    ) -> Response<Response<TransactionApplied<Call_>, Error>, Error> {
        let backend = self.backend.clone();
        future03_compat(async move {
            let tx_applied_future = backend.submit(transaction.extrinsic).await?;
            Ok(future03_compat(async move {
                let tx_applied = tx_applied_future.await?;
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
            }))
        })
    }

    fn submit<Call_: Call>(
        &self,
        author: &ed25519::Pair,
        call: Call_,
    ) -> Response<Response<TransactionApplied<Call_>, Error>, Error> {
        let account_id = author.public();
        let key_pair = author.clone();
        let genesis_hash = self.genesis_hash();
        let client = self.clone();
        future03_compat(async move {
            let nonce = client.account_nonce(&account_id).compat().await?;
            let transaction = Transaction::new_signed(
                &key_pair,
                call,
                TransactionExtra {
                    nonce,
                    genesis_hash,
                },
            );
            let tx_applied_fut = client.submit_transaction(transaction).compat().await?;
            Ok(future03_compat(async move {
                let tx_applied = tx_applied_fut.compat().await?;
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
            }))
        })
    }

    fn genesis_hash(&self) -> Hash {
        self.backend.get_genesis_hash()
    }

    fn account_nonce(&self, account_id: &AccountId) -> Response<Index, Error> {
        Box::new(self.fetch_map_value::<frame_system::AccountNonce<Runtime>, _, _>(*account_id))
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

/// Turn a 0.3 future into a boxed 0.1 future trait object.
fn future03_compat<'a, Ok, Error>(
    f: impl Future03<Output = Result<Ok, Error>> + 'a + Send,
) -> Box<dyn Future<Item = Ok, Error = Error> + Send + 'a> {
    Box::new(futures03::compat::Compat::new(Box::pin(f)))
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
