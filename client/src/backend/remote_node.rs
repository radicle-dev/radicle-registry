//! [backend::Backend] implementation for a remote full node
use futures01::future::Future;
use sr_primitives::traits::Hash as _;
use substrate_primitives::storage::StorageKey;

use radicle_registry_runtime::{
    opaque::Block as OpaqueBlock, AccountId, Event, Hash, Hashing, Runtime,
};
use substrate_subxt::system::SystemStore as _;

use crate::backend;
use crate::interface::*;

#[derive(Clone)]
pub struct RemoteNode {
    subxt_client: substrate_subxt::Client<Runtime>,
    genesis_hash: Hash,
}

type ExtrinsicSuccess = substrate_subxt::ExtrinsicSuccess<Runtime>;

impl RemoteNode {
    pub fn create() -> impl Future<Item = Self, Error = Error> {
        substrate_subxt::ClientBuilder::<Runtime>::new()
            .build()
            .and_then(|subxt_client| {
                subxt_client
                    .connect()
                    .and_then(|rpc| rpc.genesis_hash())
                    .map(|genesis_hash| RemoteNode {
                        subxt_client,
                        genesis_hash,
                    })
            })
    }

    /// Returns the list of events dispatched by the extrinsic.
    ///
    /// [ExtrinsicSuccess] contains the extrinsic hash and the list of all events in the block.
    /// From this list we return only those events that were dispatched by the extinsic.
    ///
    /// Requires an API call to get the block
    fn extract_events(
        &self,
        ext_success: ExtrinsicSuccess,
    ) -> impl Future<Item = Vec<Event>, Error = Error> {
        self.subxt_client
            .block(Some(ext_success.block))
            .and_then(move |maybe_signed_block| {
                let block = maybe_signed_block.unwrap().block;
                // TODO panic and explain
                extract_events(block, ext_success)
                    .ok_or_else(|| Error::from("Extrinsic not found in block"))
            })
    }
}

impl backend::Backend for RemoteNode {
    fn submit(
        &self,
        extrinsic: backend::UncheckedExtrinsic,
    ) -> Response<backend::TransactionApplied, Error> {
        let client = self.clone();
        Box::new(
            self.subxt_client
                .connect()
                .and_then(move |rpc| rpc.submit_and_watch_extrinsic(extrinsic))
                .and_then(move |ext_success| {
                    let tx_hash = ext_success.extrinsic;
                    let block = ext_success.block;
                    client.extract_events(ext_success).map(move |events| {
                        backend::TransactionApplied {
                            tx_hash,
                            block,
                            events,
                        }
                    })
                }),
        )
    }

    fn fetch(&self, key: &[u8]) -> Response<Option<Vec<u8>>, Error> {
        let key = StorageKey(Vec::from(key));
        Box::new(self.subxt_client.connect().and_then(move |rpc| {
            rpc.state
                .storage(key, None)
                .map_err(Error::from)
                .map(|maybe_data| maybe_data.map(|data| data.0))
        }))
    }

    fn get_transaction_extra(&self, account_id: &AccountId) -> Response<TransactionExtra, Error> {
        let genesis_hash = self.genesis_hash;
        Box::new(
            self.subxt_client
                .account_nonce(account_id.clone())
                .map(move |nonce| TransactionExtra {
                    nonce,
                    genesis_hash,
                }),
        )
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
