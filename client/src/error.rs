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

use failure::{Compat, Fail};
use jsonrpc_core_client::RpcError;
use parity_scale_codec::Error as CodecError;

use crate::event::EventExtractionError;

/// Error that may be returned by any of the [crate::ClientT] methods
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Decoding the received data failed
    #[error("Decoding the received data failed")]
    Codec(#[from] CodecError),

    /// Error from the underlying RPC connection
    #[error("Error from the underlying RPC connection")]
    Rpc(#[source] Compat<RpcError>),

    /// Invalid transaction
    #[error("Invalid transaction")]
    InvalidTransaction,

    /// Chain is running an incompatible runtime specification version
    #[error("Chain is running an incompatible runtime specification version {0}")]
    IncompatibleRuntimeVersion(u32),

    /// Failed to extract required events for a transaction
    #[error("Failed to extract required events for transaction {tx_hash}")]
    EventExtraction {
        error: EventExtractionError,
        tx_hash: crate::TxHash,
    },

    /// Events for a transaction are missing from a block.
    ///
    /// This indicates an internal error or a node error since we only look for events in the block
    /// that includes the transaction.
    #[error("Events for transaction {tx_hash} missing in block {block_hash}")]
    EventsMissing {
        block_hash: crate::BlockHash,
        tx_hash: crate::TxHash,
    },

    #[error("Could not obtain header of tip of best chain")]
    BestChainTipHeaderMissing,

    /// Block could not be found.
    #[error("Block {block_hash} could not be found")]
    BlockMissing { block_hash: crate::BlockHash },

    /// Invalid response from the node for the `chain.block_hash` method.
    ///
    /// The node is violating the application protocol.
    #[error("Invalid response from the node for the chain.block_hash method")]
    InvalidBlockHashResponse {
        response: sp_rpc::list::ListOrValue<Option<crate::BlockHash>>,
    },

    /// RPC subscription author.watch_extrinsic terminated prematurely.
    ///
    /// The node is violating the application protocol.
    #[error("RPC subscription author.watch_extrinsic terminated prematurely")]
    WatchExtrinsicStreamTerminated,

    /// Invalid [crate::backend::TransactionStatus] received in `author.watch_extrinsic` RPC
    /// subsription.
    ///
    /// The node is violating the application protocol.
    #[error("Invalid transaction status {tx_status:?} for transaction {tx_hash}")]
    InvalidTransactionStatus {
        tx_hash: crate::TxHash,
        tx_status: crate::backend::TransactionStatus,
    },
}

impl From<RpcError> for Error {
    fn from(error: RpcError) -> Self {
        Error::Rpc(error.compat())
    }
}
