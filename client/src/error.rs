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

use crate::message::EventExtractionError;

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

    /// Other error
    #[error("Other error: {0}")]
    Other(String),
}

impl From<RpcError> for Error {
    fn from(error: RpcError) -> Self {
        Error::Rpc(error.compat())
    }
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Error::Other(error)
    }
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error::Other(error.into())
    }
}
