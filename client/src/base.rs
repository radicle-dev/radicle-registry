use futures01::future::Future;
use parity_scale_codec::FullCodec;
use srml_support::storage::generator::{StorageMap, StorageValue};
use substrate_primitives::ed25519;
use substrate_primitives::storage::StorageKey;

use radicle_registry_runtime::{Call, Runtime};

pub type Error = substrate_subxt::Error;

pub struct Client {
    pub(crate) subxt_client: substrate_subxt::Client<Runtime>,
}

pub type ExtrinsicSuccess = substrate_subxt::ExtrinsicSuccess<Runtime>;

impl Client {
    pub fn create() -> impl Future<Item = Self, Error = Error> {
        substrate_subxt::ClientBuilder::<Runtime>::new()
            .build()
            .map(|subxt_client| Client { subxt_client })
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
        call: impl Into<Call>,
    ) -> impl Future<Item = ExtrinsicSuccess, Error = Error> {
        self.subxt_client
            .xt(key_pair.clone(), None)
            .and_then(move |xt_builder| xt_builder.set_system_call(call).submit_and_watch())
    }
}
