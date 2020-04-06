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

//! Define trait for client backends and provide emulator and remote node implementation
use futures::future::BoxFuture;

pub use radicle_registry_runtime::{Hash, Header, UncheckedExtrinsic};

use crate::interface::*;

mod emulator;
mod remote_node;
mod remote_node_with_executor;

pub use emulator::{Emulator, BLOCK_AUTHOR as EMULATOR_BLOCK_AUTHOR};
pub use remote_node::RemoteNode;
pub use remote_node_with_executor::RemoteNodeWithExecutor;

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
#[async_trait::async_trait]
pub trait Backend {
    /// Submit a signed transaction to the ledger and returns a future that resolves when the
    /// transaction has been applied and included in a block.
    async fn submit(
        &self,
        xt: UncheckedExtrinsic,
    ) -> Result<BoxFuture<'static, Result<TransactionApplied, Error>>, Error>;

    /// Fetch a value from the runtime state storage at the given block.
    async fn fetch(
        &self,
        key: &[u8],
        block_hash: Option<BlockHash>,
    ) -> Result<Option<Vec<u8>>, Error>;

    /// Fetch all keys with the given prefix from the state storage at the given block.
    async fn fetch_keys(
        &self,
        prefix: &[u8],
        block_hash: Option<BlockHash>,
    ) -> Result<Vec<Vec<u8>>, Error>;

    /// Fetch the header of the given block hash.
    /// If the block hash is `None`, fetch the header of the best chain tip.
    async fn block_header(&self, block_hash: Option<BlockHash>) -> Result<Header, Error>;

    /// Get the genesis hash of the blockchain. This must be obtained on backend creation.
    fn get_genesis_hash(&self) -> Hash;
}
