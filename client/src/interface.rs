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
pub use crate::transaction::{Transaction, TransactionExtra};

#[doc(inline)]
pub type Error = substrate_subxt::Error;

#[doc(inline)]
/// The hash of a transaction. Uniquely identifies a transaction.
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

    fn transfer(
        &self,
        key_pair: &ed25519::Pair,
        recipient: &AccountId,
        balance: Balance,
    ) -> Response<(), Error>;

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Error>;

    fn register_project(
        &self,
        author: &ed25519::Pair,
        project_params: RegisterProjectParams,
    ) -> Response<(), Error>;

    fn get_project(&self, id: ProjectId) -> Response<Option<Project>, Error>;

    fn list_projects(&self) -> Response<Vec<ProjectId>, Error>;

    fn create_checkpoint(
        &self,
        author: &ed25519::Pair,
        project_hash: H256,
        previous_checkpoint_id: Option<CheckpointId>,
    ) -> Response<CheckpointId, Error>;

    fn get_checkpoint(&self, id: CheckpointId) -> Response<Option<Checkpoint>, Error>;

    fn set_checkpoint(
        &self,
        author: &ed25519::Pair,
        params: SetCheckpointParams,
    ) -> Response<(), Error>;
}
