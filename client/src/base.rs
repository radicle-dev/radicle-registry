use futures01::future::Future;
use parity_scale_codec::FullCodec;
use srml_support::storage::generator::{StorageMap, StorageValue};
use substrate_primitives::ed25519;
use substrate_primitives::storage::StorageKey;

use radicle_registry_client_interface::CryptoPair as _;
use radicle_registry_runtime::{Call as RuntimeCall, Hash, Runtime};
use substrate_subxt::system::SystemStore as _;

/// Common client errors related to transport, encoding, and validity
pub type Error = substrate_subxt::Error;

pub struct Client {
    pub(crate) subxt_client: substrate_subxt::Client<Runtime>,
    genesis_hash: Hash,
}

pub type ExtrinsicSuccess = substrate_subxt::ExtrinsicSuccess<Runtime>;

impl Client {
    pub fn create() -> impl Future<Item = Self, Error = Error> {
        substrate_subxt::ClientBuilder::<Runtime>::new()
            .build()
            .and_then(|subxt_client| {
                subxt_client
                    .connect()
                    .and_then(|rpc| rpc.genesis_hash())
                    .map(|genesis_hash| Client {
                        subxt_client,
                        genesis_hash,
                    })
            })
    }

    pub fn fetch_value<S: StorageValue<T>, T: FullCodec>(
        &self,
    ) -> impl Future<Item = Option<T>, Error = Error> {
        let key = StorageKey(Vec::from(&S::storage_value_final_key()[..]));
        self.subxt_client.fetch::<T>(key)
    }

    pub fn fetch_map_value<S: StorageMap<Key, Value>, Key: FullCodec, Value: FullCodec>(
        &self,
        key: Key,
    ) -> impl Future<Item = Option<Value>, Error = Error> {
        let key = StorageKey(Vec::from(S::storage_map_final_key(key).as_ref()));
        self.subxt_client.fetch::<Value>(key)
    }

    pub fn submit_and_watch_call(
        &self,
        key_pair: &ed25519::Pair,
        call: RuntimeCall,
    ) -> impl Future<Item = ExtrinsicSuccess, Error = Error> {
        let genesis_hash = self.genesis_hash;
        let call = call.into();
        let key_pair = key_pair.clone();
        let account_id = key_pair.public().clone();
        let subxt_client = self.subxt_client.clone();
        self.subxt_client
            .account_nonce(account_id)
            .and_then(move |nonce| {
                let xt = radicle_registry_client_common::signed_extrinsic(
                    &key_pair,
                    call,
                    nonce,
                    genesis_hash,
                );
                subxt_client
                    .connect()
                    .and_then(move |rpc| rpc.submit_and_watch_extrinsic(xt))
            })
    }
}
