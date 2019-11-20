//! Define trait for client backends and provide emulator and remote node implementation
use radicle_registry_runtime::Hash;

pub use radicle_registry_runtime::UncheckedExtrinsic;

use crate::interface::*;

pub mod emulator;
pub mod remote_node;

/// Indicator that a transaction has been included in a block and has run in the runtime.
///
/// Obtained after a transaction has been submitted and processed.
pub struct TransactionApplied {
    pub tx_hash: TxHash,
    /// The hash of the block the transaction is included in.
    pub block: Hash,
    /// Events emitted by this transaction
    pub events: Vec<Event>,
}

/// Backend for talking to the ledger on a block chain.
///
/// The interface is low-level and mostly agnostic of the runtime code. Transaction extra data and
/// event information from the runtime marks an exception
pub trait Backend {
    /// Submit a signed transaction to the ledger and return when it has been applied and included
    /// in a block.
    fn submit(&self, xt: UncheckedExtrinsic) -> Response<TransactionApplied, Error>;

    /// Fetch a value from the runtime state storage.
    fn fetch(&self, key: &[u8]) -> Response<Option<Vec<u8>>, Error>;

    /// Get transaction extra from current ledger state to create valid transactions.
    fn get_transaction_extra(&self, account_id: &AccountId) -> Response<TransactionExtra, Error>;
}
