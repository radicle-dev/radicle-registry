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

use crate::{Hash, Moment};
use parity_scale_codec::Encode;
#[cfg(feature = "std")]
use parity_scale_codec::{DecodeAll, Error};
#[cfg(feature = "std")]
use sp_runtime::Digest;
use sp_runtime::{ConsensusEngineId, DigestItem};

const CONSENSUS_ID: ConsensusEngineId = *b"time";

#[cfg(feature = "std")]
pub fn load(digest: &Digest<Hash>) -> Option<Result<Moment, Error>> {
    digest
        .log(|item| match item {
            DigestItem::Consensus(CONSENSUS_ID, encoded) => Some(encoded),
            _ => None,
        })
        .map(|encoded| DecodeAll::decode_all(encoded))
}

pub fn digest_item(timestamp: Moment) -> DigestItem<Hash> {
    DigestItem::Consensus(CONSENSUS_ID, timestamp.encode())
}
