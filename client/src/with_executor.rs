//! Provides [ClientT] that uses owned [Future] executor.
use futures01::prelude::*;

use crate::*;

/// An adapter for [Client] that implements [ClientT] and spawns all futures in an owned executor.
///
/// Makes it possible to call `.wait()` when [Future] based API are not supported.
pub struct ClientWithExecutor {
    client: Client,
    runtime: tokio::runtime::Runtime,
}

impl ClientWithExecutor {
    pub fn create() -> Result<Self, Error> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let client = run_sync(&runtime, Client::create())?;
        Ok(ClientWithExecutor { client, runtime })
    }

    fn run_sync<T, F>(&self, f: impl FnOnce(&Client) -> F) -> Response<T, Error>
    where
        F: Future<Item = T, Error = Error> + Send + 'static,
        T: Send + 'static,
    {
        Box::new(run_sync(&self.runtime, f(&self.client)).into_future())
    }
}

impl ClientT for ClientWithExecutor {
    /// Sign and submit a ledger call as a transaction to the blockchain. Returns the hash of the
    /// transaction once it has been included in a block.
    fn submit<Call_: Call>(
        &self,
        author: &ed25519::Pair,
        call: Call_,
    ) -> Response<TransactionApplied<Call_>, Error> {
        self.run_sync(move |client| client.submit(author, call))
    }

    fn free_balance(&self, account_id: &AccountId) -> Response<Balance, Error> {
        self.run_sync(move |client| client.free_balance(account_id))
    }

    fn get_project(&self, id: ProjectId) -> Response<Option<Project>, Error> {
        self.run_sync(move |client| client.get_project(id))
    }

    fn list_projects(&self) -> Response<Vec<ProjectId>, Error> {
        self.run_sync(move |client| client.list_projects())
    }

    fn get_checkpoint(&self, id: CheckpointId) -> Response<Option<Checkpoint>, Error> {
        self.run_sync(move |client| client.get_checkpoint(id))
    }
}

/// Spawn the future in the given runtime and wait for the result.
fn run_sync<T, E>(
    runtime: &tokio::runtime::Runtime,
    f: impl Future<Item = T, Error = E> + Send + 'static,
) -> Result<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    let (sender, receiver) = futures01::sync::oneshot::channel();
    runtime.executor().spawn(f.then(|res| {
        // Ignore errors: We don’t care if the receiver was dropped
        sender.send(res).map_err(|_| ())
    }));
    receiver
        .then(|res| match res {
            Ok(value) => value,
            Err(_err) => panic!("ClientWithExecutor: sender was dropped"),
        })
        .wait()
}
