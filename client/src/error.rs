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

use jsonrpc_core_client::RpcError;
use parity_scale_codec::Error as CodecError;
use sp_runtime::DispatchError;

/// Error that may be returned by any of the [crate::ClientT] methods.
#[derive(Debug, derive_more::From, derive_more::Display, derive_more::TryInto)]
pub enum Error {
    /// Decoding data received from failed.
    Codec(CodecError),
    /// Error from the underlying RPC connection.
    Rpc(RpcError),
    /// Dispatch error from Substrate.
    #[display(fmt = "{:?}", "_0")]
    Dispatch(DispatchError),
    /// Invalid transaction
    InvalidTransaction(),
    /// Other error.
    Other(String),
}

impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Error::Other(error.into())
    }
}
