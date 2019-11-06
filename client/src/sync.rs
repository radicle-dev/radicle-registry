use futures01::prelude::*;

use crate::*;

/// Blocking client that has the same API as [Client] but blocks instead of returning [Future].
///
/// Asynchronous work is handled by a separate [tokio::runtime::Runtime].
pub struct SyncClient {
    client: Client,
    runtime: tokio::runtime::Runtime,
}

impl SyncClient {
    pub fn create() -> Result<Self, Error> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let client = run_sync(&runtime, Client::create())?;
        Ok(SyncClient { client, runtime })
    }

    pub fn transfer(
        &self,
        key_pair: &ed25519::Pair,
        receiver: &AccountId,
        balance: Balance,
    ) -> Result<(), Error> {
        self.run_sync(move |client| client.transfer(key_pair, receiver, balance))
    }

    pub fn free_balance(&self, account_id: &AccountId) -> Result<Balance, Error> {
        self.run_sync(move |client| client.free_balance(account_id))
    }

    pub fn register_project(
        &self,
        author: &ed25519::Pair,
        project_params: RegisterProjectParams,
    ) -> Result<(), Error> {
        self.run_sync(move |client| client.register_project(author, project_params))
    }

    pub fn create_checkpoint(
        &self,
        author: &ed25519::Pair,
        project_hash: H256,
        prev_cp: Option<CheckpointId>,
    ) -> Result<CheckpointId, Error> {
        self.run_sync(move |client| client.create_checkpoint(author, project_hash, prev_cp))
    }

    pub fn get_project(&self, id: ProjectId) -> Result<Option<Project>, Error> {
        self.run_sync(move |client| client.get_project(id))
    }

    pub fn list_projects(&self) -> Result<Vec<ProjectId>, Error> {
        self.run_sync(move |client| client.list_projects())
    }

    pub fn get_checkpoint(&self, id: CheckpointId) -> Result<Option<Checkpoint>, Error> {
        self.run_sync(move |client| client.get_checkpoint(id))
    }

    fn run_sync<T, F>(&self, f: impl FnOnce(&Client) -> F) -> Result<T, Error>
    where
        F: Future<Item = T, Error = Error> + Send + 'static,
        T: Send + 'static,
    {
        run_sync(&self.runtime, f(&self.client))
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
            Err(_err) => panic!("SyncClient: sender was dropped"),
        })
        .wait()
}
