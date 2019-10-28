use futures01::prelude::*;

use radicle_registry_runtime::{balances, counter, Address};
use substrate_subxt::balances::BalancesStore;

mod base;

#[doc(inline)]
pub use radicle_registry_runtime::{counter::CounterValue, Balance};

pub use substrate_primitives::crypto::Pair;
pub use substrate_primitives::ed25519;

#[doc(inline)]
pub use base::Error;

/// Client to interact with the radicle registry ledger through a local node.
pub struct Client {
    base_client: base::Client,
}

impl Client {
    /// Connects to a registry node running on localhost and returns a [Client].
    ///
    /// Fails if it cannot connect to a node.
    pub fn create() -> impl Future<Item = Self, Error = Error> {
        base::Client::create().map(|base_client| Client { base_client })
    }

    pub fn transfer(
        &self,
        key_pair: &ed25519::Pair,
        receiver: &ed25519::Public,
        balance: Balance,
    ) -> impl Future<Item = (), Error = Error> {
        self.base_client
            .submit_and_watch_call(
                key_pair,
                balances::Call::transfer(Address::Id(receiver.clone()), balance),
            )
            .map(|_| ())
    }

    pub fn free_balance(
        &self,
        account_id: &ed25519::Public,
    ) -> impl Future<Item = Balance, Error = Error> {
        self.base_client
            .subxt_client
            .free_balance(account_id.clone())
    }

    pub fn counter_inc(&self, key_pair: &ed25519::Pair) -> impl Future<Item = (), Error = Error> {
        self.base_client
            .submit_and_watch_call(key_pair, counter::Call::inc())
            .map(|_| ())
    }

    pub fn get_counter(&self) -> impl Future<Item = Option<CounterValue>, Error = Error> {
        self.base_client.fetch_value::<counter::Value, _>()
    }
}
