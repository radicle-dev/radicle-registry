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

//! [backend::Backend] implementation for a remote full node
use futures03::compat::Future01CompatExt as _;
use sr_primitives::traits::Hash as _;
use substrate_primitives::storage::StorageKey;

use radicle_registry_runtime::{opaque::Block as OpaqueBlock, Event, Hash, Hashing, Runtime};

use crate::backend;
use crate::interface::*;

#[derive(Clone)]
pub struct RemoteNode {
    subxt_client: substrate_subxt::Client<Runtime>,
    genesis_hash: Hash,
}

type ExtrinsicSuccess = substrate_subxt::ExtrinsicSuccess<Runtime>;

impl RemoteNode {
    pub async fn create() -> Result<Self, Error> {
        let subxt_client = substrate_subxt::ClientBuilder::<Runtime>::new()
            .build()
            .compat()
            .await?;
        let rpc = subxt_client.connect().compat().await?;
        let genesis_hash = rpc.genesis_hash().compat().await?;
        Ok(RemoteNode {
            subxt_client,
            genesis_hash,
        })
    }

    /// Returns the list of events dispatched by the extrinsic.
    ///
    /// [ExtrinsicSuccess] contains the extrinsic hash and the list of all events in the block.
    /// From this list we return only those events that were dispatched by the extinsic.
    ///
    /// Requires an API call to get the block
    async fn extract_events(&self, ext_success: ExtrinsicSuccess) -> Result<Vec<Event>, Error> {
        let maybe_signed_block = self
            .subxt_client
            .block(Some(ext_success.block))
            .compat()
            .await?;
        let block = maybe_signed_block.unwrap().block;
        // TODO panic and explain
        extract_events(block, ext_success)
            .ok_or_else(|| Error::from("Extrinsic not found in block"))
    }
}

#[async_trait::async_trait]
impl backend::Backend for RemoteNode {
    async fn submit(
        &self,
        extrinsic: backend::UncheckedExtrinsic,
    ) -> Result<backend::TransactionApplied, Error> {
        let rpc = self.subxt_client.connect().compat().await?;
        let ext_success = rpc.submit_and_watch_extrinsic(extrinsic).compat().await?;
        let tx_hash = ext_success.extrinsic;
        let block = ext_success.block;
        let events = self.extract_events(ext_success).await?;
        Ok(backend::TransactionApplied {
            tx_hash,
            block,
            events,
        })
    }

    async fn fetch(
        &self,
        key: &[u8],
        block_hash: Option<BlockHash>,
    ) -> Result<Option<Vec<u8>>, Error> {
        let key = StorageKey(Vec::from(key));
        let rpc = self.subxt_client.connect().compat().await?;
        let maybe_data = rpc.state.storage(key, block_hash).compat().await?;
        Ok(maybe_data.map(|data| data.0))
    }

    fn get_genesis_hash(&self) -> Hash {
        self.genesis_hash
    }
}

/// Given an [ExtrinsicSuccess] struct for a transaction and the block the includes the transaction
/// return all the events belonging to the transaction.
///
/// Returns `None` if no events for the transaction were found. This should be treated as an error
/// since the events should at least include the system event for the transaction.
fn extract_events(block: OpaqueBlock, ext_success: ExtrinsicSuccess) -> Option<Vec<Event>> {
    let xt_index = block
        .extrinsics
        .iter()
        .enumerate()
        .find_map(|(index, tx)| {
            if Hashing::hash_of(tx) == ext_success.extrinsic {
                Some(index)
            } else {
                None
            }
        })?;
    let events = ext_success
        .events
        .iter()
        .filter_map(|event_record| match event_record.phase {
            paint_system::Phase::ApplyExtrinsic(i) if i == xt_index as u32 => {
                Some(event_record.event.clone())
            }
            _ => None,
        })
        .collect();
    Some(events)
}
