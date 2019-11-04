//! Provide an abstract trait for registry clients and the necessary types.
//!
//! The [Client] trait defines one method for each transaction of the registry ledger as well as
//! methods to get the ledger state.
//!
//! [radicle_registry_client_interface] provides a [Client] implementation that talks to a running node.
//! [radicle_registry_memory_client] provides an implementation that runs the ledger in memory and
//! can be used for testing.
use futures::prelude::*;

pub use radicle_registry_runtime::{
    registry::{Project, ProjectId},
    AccountId, Balance,
};
pub use substrate_primitives::crypto::{Pair as CryptoPair, Public as CryptoPublic};
pub use substrate_primitives::ed25519;

#[derive(Clone, Debug)]
pub struct RegisterProjectParams {
    pub description: String,
    pub name: String,
    pub img_url: String,
}

/// Return type for all [Client] methods.
pub type Response<T, Error> = Box<dyn Future<Item = T, Error = Error> + Send>;

/// Trait for ledger clients sending transactions and looking up state.
///
/// All methods return `Response<T, Client::Error>` where.
pub trait Client {
    /// Common errors for all methods.
    ///
    /// In implementations these might be connection errors, decoding errors, etc.
    type Error;

    fn transfer(
        &self,
        key_pair: &ed25519::Pair,
        receiver: &AccountId,
        balance: Balance,
    ) -> Response<(), Self::Error>;

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Self::Error>;

    fn register_project(
        &self,
        author: &ed25519::Pair,
        project_params: RegisterProjectParams,
    ) -> Response<ProjectId, Self::Error>;

    fn get_project(&self, id: ProjectId) -> Response<Option<Project>, Self::Error>;

    fn list_projects(&self) -> Response<Vec<ProjectId>, Self::Error>;
}
