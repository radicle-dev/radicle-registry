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

//! Provides blockchain data type.

pub use radicle_registry_runtime::{Hash, Header};
use sp_runtime::OpaqueExtrinsic;

/// Block with an opaque blob of bytes as extrinsics.
///
/// In contrast to [radicle_registry_runtime::Block] serialization is stable for this type even if
/// the extrinsic type changes. This is because the extrinsic type here is [OpaqueExtrinsic], which
/// can contain all past and future versions of [radicle_registry_runtime::UncheckedExtrinsic] as
/// their serialization.
///
/// It is safe to deserialize [Block] from a serialized [radicle_registry_runtime::Block].
pub type Block = sp_runtime::generic::Block<Header, OpaqueExtrinsic>;
