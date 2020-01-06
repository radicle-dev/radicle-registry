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
use futures01::prelude::Stream as _;
use futures03::compat::{Future01CompatExt as _, Stream01CompatExt as _};
use futures03::prelude::*;
use parity_scale_codec::{Decode, Encode as _};
use sr_primitives::traits::Hash as _;
use substrate_primitives::{storage::StorageKey, twox_128};
use substrate_transaction_graph::watcher::Status as TxStatus;

use radicle_registry_runtime::{
    opaque::Block as OpaqueBlock, Event, EventRecord, Hash, Hashing, Runtime,
};

use crate::backend::{self, Backend};
use crate::interface::*;

#[derive(Clone)]
pub struct RemoteNode {
    subxt_client: substrate_subxt::Client<Runtime>,
    genesis_hash: Hash,
}

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

    /// Submit a transaction and return the block hash once it is included in a block.
    async fn submit_transaction(
        &self,
        xt: backend::UncheckedExtrinsic,
    ) -> Result<BlockHash, Error> {
        let rpc = self.subxt_client.connect().compat().await?;

        let tx_status_stream = rpc
            .author
            .watch_extrinsic(xt.encode().into())
            .compat()
            .await?;

        let mut tx_status_stream = tx_status_stream.map_err(Error::from).compat();

        loop {
            let opt_tx_status = tx_status_stream.try_next().await?;
            match opt_tx_status {
                None => return Err(Error::from("watch_extrinsic stream terminated")),
                Some(tx_status) => match tx_status {
                    TxStatus::Future | TxStatus::Ready | TxStatus::Broadcast(_) => continue,
                    TxStatus::Finalized(block_hash) => return Ok(block_hash),
                    TxStatus::Usurped(_) => return Err("Extrinsic Usurped".into()),
                    TxStatus::Dropped => return Err("Extrinsic Dropped".into()),
                    TxStatus::Invalid => return Err("Extrinsic Invalid".into()),
                },
            }
        }
    }

    /// Return all the events belonging to the transaction included in the given block.
    ///
    /// This requires the transaction to be included in the given block.
    async fn get_transaction_events(
        &self,
        tx_hash: TxHash,
        block_hash: BlockHash,
    ) -> Result<Vec<Event>, Error> {
        let events_key = b"System Events";
        let storage_key = twox_128(events_key);
        let events_data = self
            .fetch(&storage_key[..], Some(block_hash))
            .await?
            .unwrap_or_default();
        let event_records: Vec<radicle_registry_runtime::EventRecord> =
            Decode::decode(&mut &events_data[..]).map_err(Error::Codec)?;

        let rpc = self.subxt_client.connect().compat().await?;
        let opt_signed_block = rpc.chain.block(Some(block_hash)).compat().await?;
        let block = opt_signed_block
            .expect("Block that should include submitted transaction does not exist")
            .block;
        Ok(extract_transaction_events(tx_hash, block, event_records)
            .expect("Failed to extract transaction events"))
    }
}

#[async_trait::async_trait]
impl backend::Backend for RemoteNode {
    async fn submit(
        &self,
        xt: backend::UncheckedExtrinsic,
    ) -> Result<backend::TransactionApplied, Error> {
        let tx_hash = Hashing::hash_of(&xt);
        let block_hash = self.submit_transaction(xt).await?;
        let events = self.get_transaction_events(tx_hash, block_hash).await?;
        Ok(backend::TransactionApplied {
            tx_hash,
            block: block_hash,
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

/// Return all the events belonging to the transaction included in the given block.
///
/// The following conditions must hold:
/// * The transaction with `tx_hash` must be included in `block`.
/// * `event_records` are the events deposited by the runtime when `block` was executed.
///
/// Returns `None` if no events for the transaction were found. This should be treated as an error
/// since the events should at least include the system event for the transaction.
fn extract_transaction_events(
    tx_hash: TxHash,
    block: OpaqueBlock,
    event_records: Vec<EventRecord>,
) -> Option<Vec<Event>> {
    let xt_index = block
        .extrinsics
        .iter()
        .enumerate()
        .find_map(|(index, tx)| {
            if Hashing::hash_of(tx) == tx_hash {
                Some(index)
            } else {
                None
            }
        })?;
    let events = event_records
        .into_iter()
        .filter_map(|event_record| match event_record.phase {
            paint_system::Phase::ApplyExtrinsic(i) if i == xt_index as u32 => {
                Some(event_record.event)
            }
            _ => None,
        })
        .collect();
    Some(events)
}
