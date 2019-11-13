//! Run the registry ledger in memory.
//!
//! This crate provides an implementation of the registry [Client] that uses the native runtime
//! code with in-memory state. This allows you to use the ledger logic without running a node.
//!
//! The crate re-exports all items from [radicle_registry_client_interface]. You need to import the
//! [Client] trait to access the client methods.
//!
//! ```rust
//! # use futures01::prelude::*;
//! use radicle_registry_memory_client::{
//!     H256, MemoryClient, Client, CryptoPair, RegisterProjectParams, ed25519
//! };
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
//! let project_id = ("NAME".to_string(), "DOMAIN".to_string());
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
//! let project = client.get_project(project_id).wait().unwrap().unwrap();
//! assert_eq!(project.id, ("NAME".to_string(), "DOMAIN".to_string()));
//! assert_eq!(project.description, "DESCRIPTION");
//! assert_eq!(project.img_url, "IMG_URL");
//! ```

use futures01::future;
use std::sync::Mutex;

use sr_primitives::{traits::Hash as _, BuildStorage as _};
use srml_support::storage::{StorageMap as _, StorageValue as _};

use radicle_registry_runtime::{
    balances, registry, Executive, GenesisConfig, Hash, Hashing, Runtime,
};

pub use radicle_registry_client_interface::*;

/// [Client] implementation using native runtime code and in memory state through
/// [sr_io::TestExternalities].
///
/// The responses returned from the client never result in an [Error].
pub struct MemoryClient {
    test_ext: Mutex<sr_io::TestExternalities>,
    genesis_hash: Hash,
}

impl MemoryClient {
    pub fn new() -> Self {
        let genesis_config = GenesisConfig {
            srml_aura: None,
            srml_balances: None,
            srml_sudo: None,
            system: None,
        };
        let mut test_ext = sr_io::TestExternalities::new(genesis_config.build_storage().unwrap());
        let genesis_hash = test_ext.execute_with(|| {
            srml_system::Module::<Runtime>::initialize(
                &1,
                &[0u8; 32].into(),
                &[0u8; 32].into(),
                &Default::default(),
            );
            srml_system::Module::<Runtime>::block_hash(0)
        });
        MemoryClient {
            test_ext: Mutex::new(test_ext),
            genesis_hash,
        }
    }

    /// Run substrate runtime code in the test environment associated with this client.
    ///
    /// This is safe (with respect to [RefCell::borrow_mut]) as long as `f` does not call
    /// [Client::run] recursively.
    fn run<T: Send + 'static>(&self, f: impl FnOnce() -> T) -> Response<T, Error> {
        // We panic on poison errors
        let test_ext = &mut self.test_ext.lock().unwrap();
        let result = test_ext.execute_with(f);
        Box::new(future::ok(result))
    }
}

impl Client for MemoryClient {
    fn submit(&self, key_pair: &ed25519::Pair, call: Call) -> Response<TxHash, Error> {
        self.run(move || {
            let nonce = srml_system::Module::<Runtime>::account_nonce(key_pair.public());
            let runtime_call = radicle_registry_client_common::into_runtime_call(call);
            let extrinsic = radicle_registry_client_common::signed_extrinsic(
                key_pair,
                runtime_call,
                nonce,
                self.genesis_hash,
            );
            let xt_hash = Hashing::hash_of(&extrinsic);
            Executive::apply_extrinsic(extrinsic).unwrap().unwrap();
            xt_hash
        })
    }

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Error> {
        self.run(|| balances::Module::<Runtime>::free_balance(account_id))
    }

    fn get_project(&self, id: ProjectId) -> Response<Option<Project>, Error> {
        self.run(|| registry::store::Projects::get(id))
    }

    fn list_projects(&self) -> Response<Vec<ProjectId>, Error> {
        self.run(registry::store::ProjectIds::get)
    }

    fn get_checkpoint(&self, id: CheckpointId) -> Response<Option<Checkpoint>, Error> {
        self.run(|| registry::store::Checkpoints::get(id))
    }
}
