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
//! using [ClientT::account_nonce] and [ClientT::genesis_hash]. See [Transaction] for more details.
use std::sync::Arc;

use parity_scale_codec::{Decode, FullCodec};

use frame_support::storage::generator::{StorageMap, StorageValue};
use frame_support::storage::StoragePrefixedMap;
use radicle_registry_runtime::{store, store::DecodeKey as _};

mod backend;
mod error;
mod event;
mod interface;
pub mod message;
mod transaction;

pub use crate::interface::*;
pub use radicle_registry_core::{state, Balance};
pub use radicle_registry_runtime::fees::MINIMUM_FEE;
pub use radicle_registry_runtime::registry::{
    REGISTER_MEMBER_DEPOSIT, REGISTER_ORG_DEPOSIT, REGISTER_PROJECT_DEPOSIT, REGISTER_USER_DEPOSIT,
};

pub use backend::{EmulatorControl, EMULATOR_BLOCK_AUTHOR};

/// Client to interact with the radicle registry ledger via an implementation of [ClientT].
///
/// The client can either use a full node as the backend (see [Client::create]) or emulate the
/// registry in memory with [Client::new_emulator].
#[derive(Clone)]
pub struct Client {
    backend: Arc<dyn backend::Backend + Sync + Send>,
}

impl Client {
    /// Connects to a registry node running on the given host and returns a [Client].
    ///
    /// Fails if it cannot connect to a node. Uses websocket over port 9944.
    pub async fn create(host: url::Host) -> Result<Self, Error> {
        let backend = backend::RemoteNode::create(host).await?;
        Ok(Self::new(backend))
    }

    /// Same as [Client::create] but calls to the client spawn futures in an executor owned by the
    /// client.
    ///
    /// This makes it possible to call block on future in the client even if that function is
    /// called in an event loop of another executor.
    pub async fn create_with_executor(host: url::Host) -> Result<Self, Error> {
        let backend = backend::RemoteNodeWithExecutor::create(host).await?;
        Ok(Self::new(backend))
    }

    /// Create a new client that emulates the registry ledger in memory. Also returns a control
    /// handle to manipulate the emulator state. See [backend::Emulator] and [EmulatorControl] for
    /// details.
    pub fn new_emulator() -> (Self, EmulatorControl) {
        let emulator = backend::Emulator::new();
        let control = emulator.control();
        let client = Self::new(emulator);
        (client, control)
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
    #[allow(dead_code)]
    async fn fetch_value<S: StorageValue<Value>, Value: FullCodec + Send + 'static>(
        &self,
    ) -> Result<S::Query, Error>
    where
        S::Query: Send + 'static,
    {
        let backend = self.backend.clone();
        let key = S::storage_value_final_key();
        let maybe_data = backend.fetch(&key, None).await?;
        let value = match maybe_data {
            Some(data) => {
                let value =
                    Decode::decode(&mut &data[..]).map_err(|error| Error::StateDecoding {
                        error,
                        key: key.to_vec(),
                    })?;
                Some(value)
            }
            None => None,
        };
        Ok(S::from_optional_value_to_query(value))
    }

    /// Fetch a value from a map in the state storage based on a [StorageMap] implementation
    /// provided by the runtime.
    ///
    /// ```ignore
    /// client.fetch_map_value::<frame_system::AccountNonce<Runtime>, _, _>(account_id);
    /// ```
    async fn fetch_map_value<
        S: StorageMap<Key, Value>,
        Key: FullCodec,
        Value: FullCodec + Send + 'static,
    >(
        &self,
        key: Key,
    ) -> Result<S::Query, Error>
    where
        S::Query: Send + 'static,
    {
        let backend = self.backend.clone();
        // We cannot move this code into the async block. The compiler complains about a processing
        // cycle (E0391)
        let key = S::storage_map_final_key(key);
        let maybe_data = backend.fetch(&key, None).await?;
        let value = match maybe_data {
            Some(data) => {
                let value = Decode::decode(&mut &data[..])
                    .map_err(|error| Error::StateDecoding { error, key })?;
                Some(value)
            }
            None => None,
        };
        Ok(S::from_optional_value_to_query(value))
    }
}

#[async_trait::async_trait]
impl ClientT for Client {
    async fn submit_transaction<Message_: Message>(
        &self,
        transaction: Transaction<Message_>,
    ) -> Result<Response<TransactionIncluded<Message_>, Error>, Error> {
        let backend = self.backend.clone();
        let tx_included_future = backend.submit(transaction.extrinsic).await?;
        Ok(Box::pin(async move {
            let tx_included = tx_included_future.await?;
            let events = tx_included.events;
            let tx_hash = tx_included.tx_hash;
            let block = tx_included.block;
            let result = Message_::result_from_events(events)
                .map_err(|error| Error::EventExtraction { error, tx_hash })?;
            Ok(TransactionIncluded {
                tx_hash,
                block,
                result,
            })
        }))
    }

    async fn sign_and_submit_message<Message_: Message>(
        &self,
        author: &ed25519::Pair,
        message: Message_,
        fee: Balance,
    ) -> Result<Response<TransactionIncluded<Message_>, Error>, Error> {
        let account_id = author.public();
        let key_pair = author.clone();
        let genesis_hash = self.genesis_hash();
        let client = self.clone();
        let nonce = client.account_nonce(&account_id).await?;
        let runtime_transaction_version = self.runtime_version().await?.transaction_version;
        let transaction = Transaction::new_signed(
            &key_pair,
            message,
            TransactionExtra {
                nonce,
                genesis_hash,
                fee,
                runtime_transaction_version,
            },
        );
        client.submit_transaction(transaction).await
    }

    async fn block_header(&self, block_hash: BlockHash) -> Result<Option<BlockHeader>, Error> {
        self.backend.block_header(Some(block_hash)).await
    }

    async fn block_header_best_chain(&self) -> Result<BlockHeader, Error> {
        let maybe_header = self.backend.block_header(None).await?;
        maybe_header.ok_or_else(|| Error::BestChainTipHeaderMissing)
    }

    fn genesis_hash(&self) -> Hash {
        self.backend.get_genesis_hash()
    }

    async fn account_nonce(
        &self,
        account_id: &AccountId,
    ) -> Result<state::AccountTransactionIndex, Error> {
        let account_info = self
            .fetch_map_value::<store::Account, _, _>(*account_id)
            .await?;
        Ok(account_info.nonce)
    }

    async fn free_balance(&self, account_id: &AccountId) -> Result<state::AccountBalance, Error> {
        let account_info = self
            .fetch_map_value::<store::Account, _, _>(*account_id)
            .await?;
        Ok(account_info.data.free)
    }

    async fn get_org(&self, id: Id) -> Result<Option<state::Orgs1Data>, Error> {
        self.fetch_map_value::<store::Orgs1, _, _>(id.clone()).await
    }

    async fn list_orgs(&self) -> Result<Vec<Id>, Error> {
        let orgs_prefix = store::Orgs1::final_prefix();
        let keys = self.backend.fetch_keys(&orgs_prefix, None).await?;
        let mut org_ids: Vec<Id> = Vec::with_capacity(keys.len());
        for key in keys {
            let org_id = store::Orgs1::decode_key(&key)
                .expect("Invalid runtime state key. Cannot extract org ID");
            org_ids.push(org_id)
        }
        Ok(org_ids)
    }

    async fn get_user(&self, id: Id) -> Result<Option<state::Users1Data>, Error> {
        self.fetch_map_value::<store::Users1, _, _>(id.clone())
            .await
    }

    async fn list_users(&self) -> Result<Vec<Id>, Error> {
        let users_prefix = store::Users1::final_prefix();
        let keys = self.backend.fetch_keys(&users_prefix, None).await?;
        let mut user_ids: Vec<Id> = Vec::with_capacity(keys.len());
        for key in keys {
            let user_id = store::Users1::decode_key(&key)
                .expect("Invalid runtime state key. Cannot extract user ID");
            user_ids.push(user_id);
        }

        Ok(user_ids)
    }

    async fn get_project(
        &self,
        project_name: ProjectName,
        project_domain: ProjectDomain,
    ) -> Result<Option<state::Projects1Data>, Error> {
        let project_id = (project_name.clone(), project_domain.clone());
        self.fetch_map_value::<store::Projects1, _, _>(project_id.clone())
            .await
    }

    async fn list_projects(&self) -> Result<Vec<ProjectId>, Error> {
        let project_prefix = store::Projects1::final_prefix();
        let keys = self.backend.fetch_keys(&project_prefix, None).await?;
        let mut project_ids = Vec::with_capacity(keys.len());
        for key in keys {
            let project_id = store::Projects1::decode_key(&key)
                .expect("Invalid runtime state key. Cannot extract project ID");
            project_ids.push(project_id);
        }
        Ok(project_ids)
    }

    async fn get_checkpoint(
        &self,
        id: CheckpointId,
    ) -> Result<Option<state::Checkpoints1Data>, Error> {
        self.fetch_map_value::<store::Checkpoints1, _, _>(id).await
    }

    async fn runtime_version(&self) -> Result<RuntimeVersion, Error> {
        self.backend.runtime_version().await
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
        is_sync_send(Client::new_emulator().0);
    }
}
