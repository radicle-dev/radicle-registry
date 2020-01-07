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
use futures01::prelude::*;

pub use radicle_registry_runtime::Hash;

pub use radicle_registry_runtime::{
    registry::{
        Checkpoint, CheckpointId, CreateCheckpointParams, Event as RegistryEvent, Project,
        ProjectDomain, ProjectId, ProjectName, RegisterProjectParams, SetCheckpointParams,
        TransferFromProjectParams,
    },
    AccountId, Balance, Event, Hashing, Index, String32,
};
pub use substrate_primitives::crypto::{Pair as CryptoPair, Public as CryptoPublic};
pub use substrate_primitives::{ed25519, H256};

pub use crate::call::Call;
pub use crate::error::Error;
pub use crate::transaction::{Transaction, TransactionExtra};

/// The hash of a block. Uniquely identifies a block.
#[doc(inline)]
pub type BlockHash = Hash;

/// The hash of a transaction. Uniquely identifies a transaction.
#[doc(inline)]
pub type TxHash = Hash;

#[derive(Clone, Debug)]
pub struct TransferParams {
    pub recipient: AccountId,
    pub balance: Balance,
}

/// Result of a transaction being included in a block.
///
/// Returned after submitting an transaction to the blockchain.
#[derive(Clone, Debug)]
pub struct TransactionApplied<Call_: Call> {
    pub tx_hash: TxHash,
    /// The hash of the block the transaction is included in.
    pub block: Hash,
    /// Events emitted by this transaction
    pub events: Vec<Event>,
    /// The result of the runtime call.
    ///
    /// See [Call::result_from_events].
    pub result: Call_::Result,
}

/// Return type for all [ClientT] methods.
pub type Response<T, Error> = Box<dyn Future<Item = T, Error = Error> + Send>;

/// Trait for ledger clients sending transactions and looking up state.
pub trait ClientT {
    /// Submit a signed_transaction with a given ledger call.
    ///
    /// Succeeds if the transaction has been included in a block.
    fn submit_transaction<Call_: Call>(
        &self,
        transaction: Transaction<Call_>,
    ) -> Response<TransactionApplied<Call_>, Error>;

    /// Sign and submit a ledger call as a transaction to the blockchain.
    ///
    /// Same as [ClientT::submit_transaction] but takes care of signing the call.
    ///
    /// Succeeds if the transaction has been included in a block.
    fn submit<Call_: Call>(
        &self,
        author: &ed25519::Pair,
        call: Call_,
    ) -> Response<TransactionApplied<Call_>, Error>;

    /// Fetch the nonce for the given account from the chain state
    fn account_nonce(&self, account_id: &AccountId) -> Response<Index, Error>;

    /// Return the gensis hash of the chain we are communicating with.
    fn genesis_hash(&self) -> Hash;

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Error>;

    fn get_project(&self, id: ProjectId) -> Response<Option<Project>, Error>;

    fn list_projects(&self) -> Response<Vec<ProjectId>, Error>;

    fn get_checkpoint(&self, id: CheckpointId) -> Response<Option<Checkpoint>, Error>;
}
