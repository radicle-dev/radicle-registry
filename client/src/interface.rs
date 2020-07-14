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

pub use radicle_registry_runtime::{
    registry::Event as RegistryEvent, state, Balance, BlockNumber, Event, Hash, Header,
    RuntimeVersion,
};
pub use sp_core::crypto::{
    Pair as CryptoPair, Public as CryptoPublic, SecretStringError as CryptoError,
};
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

/// The header of a block
#[doc(inline)]
pub type BlockHeader = Header;

/// Result of a transaction being included in a block.
///
/// Returned after submitting an transaction to the blockchain.
#[derive(Clone, Debug)]
pub struct TransactionIncluded<Message_: Message> {
    pub tx_hash: TxHash,
    /// The hash of the block the transaction is included in.
    pub block: Hash,
    /// The result of the runtime message.
    ///
    /// See [Message::result_from_events].
    pub result: Result<Message_::Output, TransactionError>,
}

/// Return type for all [ClientT] methods.
pub type Response<T, Error> = BoxFuture<'static, Result<T, Error>>;

/// The availability status of an org or user Id
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IdStatus {
    /// The id is available and can be claimed
    Available,

    /// The id is curently taken by a user or by an org
    Taken,

    /// The id has been unregistered and is now retired
    Retired,
}

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
    /// let tx_included_fut = client.submit_transaction(tx).await?;
    ///
    /// // We can now wait for the transaction to be included in a block.
    /// //
    /// // This will error if the transaction becomes invalid (for example due to the nonce) or if
    /// // we fail to retrieve the transaction state from the node.
    /// //
    /// // This will not error if the transaction errored while applying. See
    /// // TransactionIncluded::result for that.
    /// let tx_included = tx_included_fut.await?;
    ///
    /// Ok(())
    /// # }
    /// ```
    ///
    /// See the `getting_started` example for more details.
    async fn submit_transaction<Message_: Message>(
        &self,
        transaction: Transaction<Message_>,
    ) -> Result<Response<TransactionIncluded<Message_>, Error>, Error>;

    /// Sign and submit a ledger message as a transaction to the blockchain.
    ///
    /// Same as [ClientT::submit_transaction] but takes care of signing the message.
    async fn sign_and_submit_message<Message_: Message>(
        &self,
        author: &ed25519::Pair,
        message: Message_,
        fee: Balance,
    ) -> Result<Response<TransactionIncluded<Message_>, Error>, Error>;

    /// Fetch the nonce for the given account from the chain state
    async fn account_nonce(
        &self,
        account_id: &AccountId,
    ) -> Result<state::AccountTransactionIndex, Error>;

    /// Fetch the header of the given block hash
    async fn block_header(&self, block_hash: BlockHash) -> Result<Option<BlockHeader>, Error>;

    /// Fetch the header of the best chain tip
    async fn block_header_best_chain(&self) -> Result<BlockHeader, Error>;

    /// Return the genesis hash of the chain we are communicating with.
    fn genesis_hash(&self) -> Hash;

    /// Get the runtime version at the latest block
    async fn runtime_version(&self) -> Result<RuntimeVersion, Error>;

    async fn free_balance(&self, account_id: &AccountId) -> Result<Balance, Error>;

    async fn get_id_status(&self, id: &Id) -> Result<IdStatus, Error>;

    async fn get_org(&self, org_id: Id) -> Result<Option<state::Orgs1Data>, Error>;

    async fn list_orgs(&self) -> Result<Vec<Id>, Error>;

    async fn get_user(&self, user_id: Id) -> Result<Option<state::Users1Data>, Error>;

    async fn list_users(&self) -> Result<Vec<Id>, Error>;

    async fn get_project(
        &self,
        project_name: ProjectName,
        project_domain: ProjectDomain,
    ) -> Result<Option<state::Projects1Data>, Error>;

    async fn list_projects(&self) -> Result<Vec<ProjectId>, Error>;

    async fn get_checkpoint(
        &self,
        id: CheckpointId,
    ) -> Result<Option<state::Checkpoints1Data>, Error>;
}
