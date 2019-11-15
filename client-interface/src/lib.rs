//! Provide an abstract trait for registry clients and the necessary types.
//!
//! The [Client] trait defines one method for each transaction of the registry ledger as well as
//! methods to get the ledger state.
//!
//! [radicle_registry_client_interface] provides a [Client] implementation that talks to a running node.
//! [radicle_registry_memory_client] provides an implementation that runs the ledger in memory and
//! can be used for testing.
use futures::prelude::*;

use radicle_registry_runtime::Hash;

pub use radicle_registry_runtime::{
    registry::{
        Checkpoint, CheckpointId, CreateCheckpointParams, Event as RegistryEvent, Project,
        ProjectDomain, ProjectId, ProjectName, RegisterProjectParams, SetCheckpointParams,
        String32, TransferFromProjectParams,
    },
    AccountId, Balance, Event, Index,
};
pub use substrate_primitives::crypto::{Pair as CryptoPair, Public as CryptoPublic};
pub use substrate_primitives::{ed25519, H256};

mod call;

pub use call::Call;

#[doc(inline)]
pub type Error = substrate_subxt::Error;

#[doc(inline)]
/// The hash of a transaction. Uniquely identifies a transaction.
pub type TxHash = Hash;

#[derive(Copy, Clone)]
/// All data that is necessary to build the [SignedPayload] for a extrinsic.
pub struct TransactionExtra {
    pub nonce: Index,
    pub genesis_hash: Hash,
}

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

/// Return type for all [Client] methods.
pub type Response<T, Error> = Box<dyn Future<Item = T, Error = Error> + Send>;

/// Trait for ledger clients sending transactions and looking up state.
pub trait Client {
    /// Sign and submit a ledger call as a transaction to the blockchain.
    ///
    /// Succeeds if the transaction has been included in a block.
    fn submit<Call_: Call>(
        &self,
        author: &ed25519::Pair,
        call: Call_,
    ) -> Response<TransactionApplied<Call_>, Error>;

    fn get_transaction_extra(&self, account_id: &AccountId) -> Response<TransactionExtra, Error>;

    fn transfer(
        &self,
        key_pair: &ed25519::Pair,
        recipient: &AccountId,
        balance: Balance,
    ) -> Response<(), Error> {
        Box::new(
            self.submit(
                key_pair,
                TransferParams {
                    recipient: recipient.clone(),
                    balance,
                },
            )
            .map(|_| ()),
        )
    }

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Error>;

    fn register_project(
        &self,
        author: &ed25519::Pair,
        project_params: RegisterProjectParams,
    ) -> Response<(), Error> {
        Box::new(self.submit(author, project_params).map(|_| ()))
    }

    fn get_project(&self, id: ProjectId) -> Response<Option<Project>, Error>;

    fn list_projects(&self) -> Response<Vec<ProjectId>, Error>;

    fn create_checkpoint(
        &self,
        author: &ed25519::Pair,
        project_hash: H256,
        previous_checkpoint: Option<CheckpointId>,
    ) -> Response<CheckpointId, Error> {
        let checkpoint_id = CheckpointId::random();
        Box::new(
            self.submit(
                author,
                CreateCheckpointParams {
                    checkpoint_id,
                    project_hash,
                    previous_checkpoint,
                },
            )
            .map(move |_| checkpoint_id),
        )
    }

    fn get_checkpoint(&self, id: CheckpointId) -> Response<Option<Checkpoint>, Error>;

    fn set_checkpoint(
        &self,
        author: &ed25519::Pair,
        params: SetCheckpointParams,
    ) -> Response<(), Error> {
        Box::new(self.submit(author, params).map(|_| ()))
    }
}
