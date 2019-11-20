//! Run the registry ledger in memory.
//!
//! This crate provides an implementation of the registry [Client] that uses the native runtime
//! code with in-memory state. This allows you to use the ledger logic without running a node. [See
//! below](#differences) for a list of differences to the real node client.
//!
//! The crate re-exports all items from [radicle_registry_client_interface]. You need to import the
//! [Client] trait to access the client methods.
//!
//! ```rust
//! # use futures01::prelude::*;
//! use radicle_registry_memory_client::{
//!     H256, MemoryClient, Client, CryptoPair, RegisterProjectParams, ed25519
//! };
//! use radicle_registry_runtime::registry::{ProjectDomain, ProjectName};
//!
//! let client = MemoryClient::new();
//! let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
//!
//! let project_hash = H256::random();
//! let checkpoint_id = client
//!     .create_checkpoint(
//!         &alice,
//!         project_hash,
//!         None
//!     )
//!     .wait()
//!     .unwrap();
//!
//!     let project_id = (
//!         ProjectName::from_string("NAME".to_string()).unwrap(),
//!         ProjectDomain::from_string("DOMAIN".to_string()).unwrap(),
//!     );
//! client
//!     .register_project(
//!         &alice,
//!         RegisterProjectParams {
//!             id: project_id.clone(),
//!             description: "DESCRIPTION".to_string(),
//!             img_url: "IMG_URL".to_string(),
//!             checkpoint_id,
//!         },
//!     )
//!     .wait()
//!     .unwrap();
//!
//! let project = client.get_project(project_id.clone()).wait().unwrap().unwrap();
//! assert_eq!(project.id, project_id);
//! assert_eq!(project.description, "DESCRIPTION");
//! assert_eq!(project.img_url, "IMG_URL");
//! ```
//!
//! # Differences
//!
//! The [MemoryClient] does not produce blocks. In particular the `blocks` field in
//! [TransactionApplied]` always equals `Hash::default` when returned from [Client::submit].

use futures01::{future, prelude::*};
use std::sync::{Arc, Mutex};

use paint_support::storage::generator::{StorageMap, StorageValue};
use parity_scale_codec::{Decode, FullCodec};
use sr_primitives::{traits::Hash as _, BuildStorage as _};

use radicle_registry_runtime::{
    balances, registry, BalancesConfig, Executive, GenesisConfig, Hash, Hashing, Runtime,
};

pub use radicle_registry_client_interface::*;

/// [Client] implementation using native runtime code and in memory state through
/// [sr_io::TestExternalities].
///
/// The responses returned from the client never result in an [Error].
#[derive(Clone)]
pub struct MemoryClient {
    test_ext: Arc<Mutex<sr_io::TestExternalities>>,
    genesis_hash: Hash,
}

impl MemoryClient {
    pub fn new() -> Self {
        let genesis_config = GenesisConfig {
            paint_aura: None,
            paint_balances: Some(BalancesConfig {
                balances: vec![(
                    ed25519::Pair::from_string("//Alice", None)
                        .unwrap()
                        .public(),
                    1 << 60,
                )],
                vesting: vec![],
            }),
            paint_sudo: None,
            system: None,
        };
        let mut test_ext = sr_io::TestExternalities::new(genesis_config.build_storage().unwrap());
        let genesis_hash = test_ext.execute_with(|| {
            paint_system::Module::<Runtime>::initialize(
                &1,
                &[0u8; 32].into(),
                &[0u8; 32].into(),
                &Default::default(),
            );
            paint_system::Module::<Runtime>::block_hash(0)
        });
        MemoryClient {
            test_ext: Arc::new(Mutex::new(test_ext)),
            genesis_hash,
        }
    }

    pub fn fetch_value<S: StorageValue<Value>, Value: FullCodec>(&self) -> Response<S::Query, Error>
    where
        S::Query: Send + 'static,
    {
        let test_ext = &mut self.test_ext.lock().unwrap();
        let result = storage_lookup(test_ext, S::storage_value_final_key());
        Box::new(
            result
                .map(S::from_optional_value_to_query)
                .map_err(Error::Codec)
                .into_future(),
        )
    }

    pub fn fetch_map_value<S: StorageMap<Key, Value>, Key: FullCodec, Value: FullCodec>(
        &self,
        key: Key,
    ) -> Response<S::Query, Error>
    where
        S::Query: Send + 'static,
    {
        let test_ext = &mut self.test_ext.lock().unwrap();
        let result = storage_lookup(test_ext, S::storage_map_final_key(key));
        Box::new(
            result
                .map(S::from_optional_value_to_query)
                .map_err(Error::Codec)
                .into_future(),
        )
    }
}

impl Client for MemoryClient {
    fn submit<Call_: Call>(
        &self,
        author: &ed25519::Pair,
        call: Call_,
    ) -> Response<TransactionApplied<Call_>, Error> {
        let account_id = author.public();
        let client = self.clone();
        let key_pair = author.clone();
        Box::new(
            self.get_transaction_extra(&account_id)
                .and_then(move |extra| {
                    let extrinsic = radicle_registry_client_common::signed_extrinsic(
                        &key_pair,
                        call.into_runtime_call(),
                        extra.nonce,
                        extra.genesis_hash,
                    );
                    let tx_hash = Hashing::hash_of(&extrinsic);
                    let test_ext = &mut client.test_ext.lock().unwrap();
                    let events = test_ext.execute_with(move || {
                        let event_start_index = paint_system::Module::<Runtime>::event_count();
                        let _apply_outcome = Executive::apply_extrinsic(extrinsic).unwrap();
                        paint_system::Module::<Runtime>::events()
                            .into_iter()
                            .skip(event_start_index as usize)
                            .map(|event_record| event_record.event)
                            .collect::<Vec<Event>>()
                    });
                    let result = Call_::result_from_events(events.clone())?;
                    Ok(TransactionApplied {
                        tx_hash,
                        block: Default::default(),
                        events,
                        result,
                    })
                }),
        )
    }

    fn get_transaction_extra(&self, account_id: &AccountId) -> Response<TransactionExtra, Error> {
        let test_ext = &mut self.test_ext.lock().unwrap();
        let nonce =
            test_ext.execute_with(|| paint_system::Module::<Runtime>::account_nonce(account_id));
        Box::new(future::ok(TransactionExtra {
            nonce,
            genesis_hash: self.genesis_hash,
        }))
    }

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Error> {
        Box::new(self.fetch_map_value::<balances::FreeBalance<Runtime>, _, _>(account_id.clone()))
    }

    fn get_project(&self, id: ProjectId) -> Response<Option<Project>, Error> {
        Box::new(self.fetch_map_value::<registry::store::Projects, _, _>(id))
    }

    fn list_projects(&self) -> Response<Vec<ProjectId>, Error> {
        Box::new(self.fetch_value::<registry::store::ProjectIds, _>())
    }

    fn get_checkpoint(&self, id: CheckpointId) -> Response<Option<Checkpoint>, Error> {
        Box::new(self.fetch_map_value::<registry::store::Checkpoints, _, _>(id))
    }
}

/// Lookup and decode a storage value in the [TestExternalities] context.
fn storage_lookup<Value: Decode>(
    test_ext: &mut sr_io::TestExternalities,
    key: impl AsRef<[u8]>,
) -> Result<Option<Value>, parity_scale_codec::Error> {
    let maybe_data = test_ext.execute_with(|| sr_io::storage::get(key.as_ref()));
    match maybe_data {
        Some(data) => Value::decode(&mut data.as_slice()).map(Some),
        None => Ok(None),
    }
}
