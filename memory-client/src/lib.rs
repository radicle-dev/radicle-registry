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
//!     MemoryClient, Client, CryptoPair, RegisterProjectParams, ed25519
//! };
//! let client = MemoryClient::new();
//! let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
//!
//! let project_id = ("NAME".to_string(), "DOMAIN".to_string());
//! client
//!     .register_project(
//!         &alice,
//!         RegisterProjectParams {
//!             id: project_id.clone(),
//!             description: "DESCRIPTION".to_string(),
//!             img_url: "IMG_URL".to_string(),
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
use std::cell::RefCell;

use sr_primitives::BuildStorage as _;
use srml_support::storage::{StorageMap as _, StorageValue as _};

use radicle_registry_runtime::{balances, registry, GenesisConfig, Origin, Runtime};

pub use radicle_registry_client_interface::*;

/// [Client] implementation using native runtime code and in memory state through
/// [sr_io::TestExternalities].
///
/// The responses returned from the client never result in an [Error].
pub struct MemoryClient {
    test_ext: RefCell<sr_io::TestExternalities>,
}

impl MemoryClient {
    pub fn new() -> Self {
        let genesis_config = GenesisConfig {
            srml_aura: None,
            srml_balances: None,
            srml_sudo: None,
            system: None,
        };
        let test_ext = sr_io::TestExternalities::new(genesis_config.build_storage().unwrap());
        MemoryClient {
            test_ext: RefCell::new(test_ext),
        }
    }

    /// Run substrate runtime code in the test environment associated with this client.
    ///
    /// This is safe (with respect to [RefCell::borrow_mut]) as long as `f` does not call
    /// [Client::run] recursively.
    fn run<T: Send + 'static>(&self, f: impl FnOnce() -> T) -> Response<T, Error> {
        let test_ext = &mut self.test_ext.borrow_mut();
        let result = test_ext.execute_with(f);
        Box::new(future::ok(result))
    }
}

impl Client for MemoryClient {
    fn transfer(
        &self,
        author: &ed25519::Pair,
        receiver: &AccountId,
        balance: Balance,
    ) -> Response<(), Error> {
        self.run(|| {
            let origin = Origin::signed(author.public());
            balances::Module::<Runtime>::transfer(origin, receiver.clone(), balance)
                .expect("origin is valid and the only possible error")
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

    fn register_project(
        &self,
        author: &ed25519::Pair,
        project_params: RegisterProjectParams,
    ) -> Response<(), Error> {
        self.run(|| {
            let origin = Origin::signed(author.public());
            registry::Module::<Runtime>::register_project(
                origin,
                registry::RegisterProjectParams {
                    id: project_params.id,
                    description: project_params.description,
                    img_url: project_params.img_url,
                },
            )
            .expect("origin is valid and the only possible error");
        })
    }
}
