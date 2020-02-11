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

//! Provide an abstract trait for the registry client and the necessary types.
//!
//! The [ClientT] trait defines one method for each transaction of the registry ledger as well as
//! methods to get the ledger state.
use futures::future::BoxFuture;

pub use radicle_registry_core::*;

pub use radicle_registry_runtime::Hash;

pub use radicle_registry_runtime::{registry::Event as RegistryEvent, Event};
pub use sp_core::crypto::{Pair as CryptoPair, Public as CryptoPublic};
pub use sp_core::{ed25519, H256};

pub use crate::error::Error;
pub use crate::message::Message;
pub use crate::transaction::{Transaction, TransactionExtra};

/// The hash of a block. Uniquely identifies a block.
#[doc(inline)]
pub type BlockHash = Hash;

/// The hash of a transaction. Uniquely identifies a transaction.
#[doc(inline)]
pub type TxHash = Hash;

/// Result of a transaction being included in a block.
///
/// Returned after submitting an transaction to the blockchain.
#[derive(Clone, Debug)]
pub struct TransactionApplied<Message_: Message> {
    pub tx_hash: TxHash,
    /// The hash of the block the transaction is included in.
    pub block: Hash,
    /// Events emitted by this transaction
    pub events: Vec<Event>,
    /// The result of the runtime message.
    ///
    /// See [Message::result_from_events].
    pub result: Message_::Result,
}

/// Return type for all [ClientT] methods.
pub type Response<T, Error> = BoxFuture<'static, Result<T, Error>>;

/// Trait for ledger clients sending transactions and looking up state.
#[async_trait::async_trait]
pub trait ClientT {
    /// Submit a signed transaction.
    ///
    /// ```no_run
    /// # use radicle_registry_client::*;
    /// # async fn example<M: Message>(client: Client, tx: Transaction<M>) -> Result<(), Error> {
    ///
    /// // Submit the transaction to the node.
    /// //
    /// // If this is successful the transaction has been accepted by the node. The node will then
    /// // dissemniate the transaction to the network.
    /// //
    /// // This call fails if the transaction is invalid or if the RPC communication with the node
    /// // failed.
    /// let applied_fut = client.submit_transaction(tx).await?;
    ///
    /// // We can now wait for the transaction to be included in a block.
    /// //
    /// // This will error if the transaction becomes invalid (for example due to the nonce) or if
    /// // we fail to retrieve the transaction state from the node.
    /// //
    /// // This will not error if the transaction errored while applying. See
    /// // TransactionApplied::result for that.
    /// let transaction_applied = applied_fut.await?;
    ///
    /// Ok(())
    /// # }
    /// ```
    ///
    /// See the `getting_started` example for more details.
    async fn submit_transaction<Message_: Message>(
        &self,
        transaction: Transaction<Message_>,
    ) -> Result<Response<TransactionApplied<Message_>, Error>, Error>;

    /// Sign and submit a ledger message as a transaction to the blockchain.
    ///
    /// Same as [ClientT::submit_transaction] but takes care of signing the message.
    async fn sign_and_submit_message<Message_: Message>(
        &self,
        author: &ed25519::Pair,
        message: Message_,
    ) -> Result<Response<TransactionApplied<Message_>, Error>, Error>;

    /// Fetch the nonce for the given account from the chain state
    async fn account_nonce(&self, account_id: &AccountId) -> Result<state::Index, Error>;

    /// Return the gensis hash of the chain we are communicating with.
    fn genesis_hash(&self) -> Hash;

    async fn free_balance(&self, account_id: &AccountId) -> Result<Balance, Error>;

    async fn get_org(&self, id: OrgId) -> Result<Option<Org>, Error>;

    async fn list_orgs(&self) -> Result<Vec<OrgId>, Error>;

    async fn get_project(&self, id: ProjectId) -> Result<Option<Project>, Error>;

    async fn list_projects(&self) -> Result<Vec<ProjectId>, Error>;

    async fn get_checkpoint(&self, id: CheckpointId) -> Result<Option<state::Checkpoint>, Error>;
}
