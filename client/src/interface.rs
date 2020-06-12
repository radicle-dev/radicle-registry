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
use core::str::FromStr;
use futures::future::BoxFuture;
use hex::ToHex;
use parity_scale_codec::Encode;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_runtime::traits::Hash as _;

pub use radicle_registry_core::*;

pub use radicle_registry_runtime::{BlockNumber, Header, RuntimeVersion};

pub use radicle_registry_runtime::{
    registry::Event as RegistryEvent, Balance, Event, Hash as RuntimeHash,
};
pub use sp_core::crypto::{Pair as CryptoPair, Public as CryptoPublic};
pub use sp_core::{ed25519, H256};

pub use crate::error::Error;
pub use crate::message::Message;
pub use crate::transaction::{Transaction, TransactionExtra};

//TODO(nuno):
// * Test (deserialize . serialize)

/// A hash of some data used by the chain.
///
/// Wraps the hash type used by the runtime, [RuntimeHash],
/// providing official Serialize and Deserialize implementations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, core::hash::Hash)]
pub struct Hash(RuntimeHash);

impl Hash {
    pub fn zero() -> Self {
        RuntimeHash::zero().into()
    }

    pub fn hash_of<E: Encode>(x: &E) -> Hash {
        radicle_registry_runtime::Hashing::hash_of(x).into()
    }

    pub fn random() -> Self {
        RuntimeHash::random().into()
    }
}

impl From<RuntimeHash> for Hash {
    fn from(h: RuntimeHash) -> Self {
        Self(h)
    }
}

impl Into<RuntimeHash> for Hash {
    fn into(self) -> RuntimeHash {
        self.0
    }
}

impl std::convert::AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl core::fmt::Display for Hash {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for Hash {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.encode_hex::<String>())
    }
}

impl<'de> Deserialize<'de> for Hash {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        let hash = RuntimeHash::from_str(s).map_err(serde::de::Error::custom)?;

        Ok(Self(hash))
    }
}

/// The hash of a block. Uniquely identifies a block.
#[doc(inline)]
pub type BlockHash = Hash;

/// The header of a block
#[doc(inline)]
pub type BlockHeader = Header;

/// Result of a transaction being included in a block.
///
/// Returned after submitting a transaction to the blockchain.
#[derive(Clone, Debug)]
pub struct TransactionIncluded<Message_: Message> {
    pub tx_hash: Hash,
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
    async fn block_header(&self, block_hash: BlockHash) -> Result<BlockHeader, Error>;

    /// Fetch the header of the best chain tip
    async fn block_header_best_chain(&self) -> Result<BlockHeader, Error>;

    /// Return the gensis hash of the chain we are communicating with.
    fn genesis_hash(&self) -> Hash;

    async fn free_balance(&self, account_id: &AccountId) -> Result<Balance, Error>;

    async fn get_org(&self, org_id: Id) -> Result<Option<Org>, Error>;

    async fn list_orgs(&self) -> Result<Vec<Id>, Error>;

    async fn get_user(&self, user_id: Id) -> Result<Option<User>, Error>;

    async fn list_users(&self) -> Result<Vec<Id>, Error>;

    async fn get_project(
        &self,
        project_name: ProjectName,
        project_domain: ProjectDomain,
    ) -> Result<Option<Project>, Error>;

    async fn list_projects(&self) -> Result<Vec<ProjectId>, Error>;

    async fn get_checkpoint(&self, id: CheckpointId) -> Result<Option<state::Checkpoint>, Error>;

    async fn onchain_runtime_version(&self) -> Result<RuntimeVersion, Error>;
}
