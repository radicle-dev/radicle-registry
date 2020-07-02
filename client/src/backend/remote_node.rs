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
use futures::compat::{Future01CompatExt as _, Stream01CompatExt as _};
use futures::future::BoxFuture;
use futures::prelude::*;
use futures01::stream::Stream as _;
use jsonrpc_core_client::RpcChannel;
use lazy_static::lazy_static;
use parity_scale_codec::Encode as _;
use sc_rpc_api::{author::AuthorClient, chain::ChainClient, state::StateClient};
use sp_core::{storage::StorageKey, twox_128};
use sp_rpc::{list::ListOrValue, number::NumberOrHex};
use sp_runtime::{generic::SignedBlock, traits::Hash as _};
use sp_transaction_pool::TransactionStatus as TxStatus;
use std::sync::Arc;
use url::Url;

use radicle_registry_runtime::{Block, BlockNumber, Hash, Hashing, Header};

use crate::backend::{self, Backend};
use crate::event::{Event, EventRecord};
use crate::interface::*;

/// Collection of substrate RPC clients
#[derive(Clone)]
struct Rpc {
    state: StateClient<BlockHash>,
    chain: ChainClient<BlockNumber, Hash, Header, SignedBlock<Block>>,
    author: AuthorClient<Hash, BlockHash>,
}

#[derive(Clone)]
pub struct RemoteNode {
    genesis_hash: Hash,
    rpc: Arc<Rpc>,
}

lazy_static! {
    static ref SYSTEM_EVENTS_STORAGE_KEY: [u8; 32] = {
        let mut events_key = [0u8; 32];
        events_key[0..16].copy_from_slice(&twox_128(b"System"));
        events_key[16..32].copy_from_slice(&twox_128(b"Events"));
        events_key
    };
}

impl RemoteNode {
    pub async fn create(host: url::Host) -> Result<Self, Error> {
        let url = Url::parse(&format!("ws://{}:9944", host)).expect("Is valid url; qed");
        let channel: RpcChannel = jsonrpc_core_client::transports::ws::connect(&url)
            .compat()
            .await?;
        let rpc = Arc::new(Rpc {
            state: channel.clone().into(),
            chain: channel.clone().into(),
            author: channel.clone().into(),
        });
        check_runtime_version(&rpc).await?;
        let genesis_hash_result = rpc
            .chain
            .block_hash(Some(NumberOrHex::Number(BlockNumber::min_value()).into()))
            .compat()
            .await?;
        let genesis_hash = match genesis_hash_result {
            ListOrValue::Value(Some(genesis_hash)) => genesis_hash,
            other => {
                return Err(Error::Other(format!(
                    "Invalid chain.block_hash result {:?}",
                    other
                )))
            }
        };
        Ok(RemoteNode { genesis_hash, rpc })
    }

    /// Submit a transaction and return the block hash once it is included in a block.
    async fn submit_transaction(
        &self,
        xt: backend::UncheckedExtrinsic,
    ) -> Result<impl Future<Output = Result<Hash, Error>>, Error> {
        let tx_status_stream = self
            .rpc
            .author
            .watch_extrinsic(xt.encode().into())
            .compat()
            .await?;

        let mut tx_status_stream = tx_status_stream.map_err(Error::from).compat();

        let opt_tx_status = tx_status_stream.try_next().await?;
        match opt_tx_status {
            None => return Err(Error::from("watch_extrinsic stream terminated")),
            Some(tx_status) => match tx_status {
                TxStatus::Future | TxStatus::Ready | TxStatus::Broadcast(_) => (),
                other => return Err(format!("Invalid TxStatus: {:?}", other).into()),
            },
        }

        Ok(async move {
            loop {
                let opt_tx_status = tx_status_stream.try_next().await?;
                match opt_tx_status {
                    None => return Err(Error::from("watch_extrinsic stream terminated")),
                    Some(tx_status) => match tx_status {
                        TxStatus::Future | TxStatus::Ready | TxStatus::Broadcast(_) => continue,
                        TxStatus::InBlock(block_hash) => return Ok(block_hash),
                        other => return Err(format!("Invalid TxStatus: {:?}", other).into()),
                    },
                }
            }
        })
    }

    /// Return all the events belonging to the transaction included in the given block.
    ///
    /// This requires the transaction to be included in the given block.
    async fn get_transaction_events(
        &self,
        tx_hash: TxHash,
        block_hash: BlockHash,
    ) -> Result<Vec<Event>, Error> {
        let runtime_spec_version = runtime_version(&self.rpc, Some(block_hash))
            .await?
            .spec_version;
        let events_data = self
            .fetch(SYSTEM_EVENTS_STORAGE_KEY.as_ref(), Some(block_hash))
            .await?
            .unwrap_or_default();
        let event_records =
            EventRecord::decode_vec(runtime_spec_version, &events_data).map_err(Error::Codec)?;

        let signed_block = self
            .rpc
            .chain
            .block(Some(block_hash))
            .compat()
            .await?
            .ok_or_else(|| {
                Error::from("Block that should include submitted transaction does not exist")
            })?;
        extract_transaction_events(tx_hash, &signed_block.block, event_records)
            .ok_or_else(|| Error::from("Failed to extract transaction events"))
    }
}

#[async_trait::async_trait]
impl backend::Backend for RemoteNode {
    async fn submit(
        &self,
        xt: backend::UncheckedExtrinsic,
    ) -> Result<BoxFuture<'static, Result<backend::TransactionIncluded, Error>>, Error> {
        let tx_hash = Hashing::hash_of(&xt);
        let block_hash_future = self.submit_transaction(xt).await?;
        let this = self.clone();

        Ok(Box::pin(async move {
            let block_hash = block_hash_future.await?;
            let events = this.get_transaction_events(tx_hash, block_hash).await?;
            Ok(backend::TransactionIncluded {
                tx_hash,
                block: block_hash,
                events,
            })
        }))
    }

    async fn fetch(
        &self,
        key: &[u8],
        block_hash: Option<BlockHash>,
    ) -> Result<Option<Vec<u8>>, Error> {
        let key = StorageKey(Vec::from(key));
        let maybe_data = self.rpc.state.storage(key, block_hash).compat().await?;
        Ok(maybe_data.map(|data| data.0))
    }

    async fn fetch_keys(
        &self,
        prefix: &[u8],
        block_hash: Option<BlockHash>,
    ) -> Result<Vec<Vec<u8>>, Error> {
        let prefix = StorageKey(Vec::from(prefix));
        let keys = self
            .rpc
            .state
            .storage_keys(prefix, block_hash)
            .compat()
            .await?;
        Ok(keys.into_iter().map(|key| key.0).collect())
    }

    async fn block_header(
        &self,
        block_hash: Option<BlockHash>,
    ) -> Result<Option<BlockHeader>, Error> {
        self.rpc
            .chain
            .header(block_hash)
            .compat()
            .await
            .map_err(Error::from)
    }

    fn get_genesis_hash(&self) -> Hash {
        self.genesis_hash
    }

    async fn runtime_version(&self) -> Result<RuntimeVersion, Error> {
        runtime_version(&self.rpc, None).await
    }
}

async fn check_runtime_version(rpc: &Rpc) -> Result<(), Error> {
    match runtime_version(rpc, None).await?.spec_version {
        9 | 10 | 11 | 12 | 13 => Ok(()),
        other => Err(Error::IncompatibleRuntimeVersion(other)),
    }
}

async fn runtime_version(
    rpc: &Rpc,
    block_hash: Option<BlockHash>,
) -> Result<RuntimeVersion, Error> {
    rpc.state
        .runtime_version(block_hash)
        .compat()
        .await
        .map_err(Into::into)
}

/// Return all the events belonging to the transaction included in the given block.
///
/// The following conditions must hold:
/// * The transaction with `tx_hash` must be included in `block`.
/// * `event_records` are the events deposited by the runtime when `block` was executed.
///
/// Returns `None` if no events for the transaction were found. This should be treated as an error
/// since the events should at least include the system event for the transaction.
pub(crate) fn extract_transaction_events(
    tx_hash: TxHash,
    block: &Block,
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
        .filter_map(|event_record| match event_record.transaction_index() {
            Some(i) if i == xt_index as u32 => Some(event_record.event()),
            _ => None,
        })
        .collect();
    Some(events)
}
