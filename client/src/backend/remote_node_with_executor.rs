// Radicle Registry
// Copyright (C) 2019 Monadic GmbH <radicle@monadic.xyz>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as
// published by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

//! Provides [RemoteNodeWithExecutor] backend
use futures01::prelude::*;
use std::sync::Arc;

use crate::backend;
use crate::interface::*;

/// Client backend that wraps [crate::backend::RemoteNode] but spawns all futures in
/// its own executor.
#[derive(Clone)]
pub struct RemoteNodeWithExecutor {
    backend: backend::RemoteNode,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl RemoteNodeWithExecutor {
    pub fn create() -> Result<Self, Error> {
        let runtime = Arc::new(tokio::runtime::Runtime::new().unwrap());
        let backend = run_sync(&runtime, backend::RemoteNode::create())?;
        Ok(RemoteNodeWithExecutor { backend, runtime })
    }

    fn run_sync<T, F>(&self, f: impl FnOnce(&backend::RemoteNode) -> F) -> Response<T, Error>
    where
        F: Future<Item = T, Error = Error> + Send + 'static,
        T: Send + 'static,
    {
        Box::new(run_sync(&self.runtime, f(&self.backend)).into_future())
    }
}

impl backend::Backend for RemoteNodeWithExecutor {
    fn submit(
        &self,
        xt: backend::UncheckedExtrinsic,
    ) -> Response<backend::TransactionApplied, Error> {
        self.run_sync(move |backend| backend.submit(xt))
    }

    /// Fetch a value from the runtime state storage.
    fn fetch(&self, key: &[u8]) -> Response<Option<Vec<u8>>, Error> {
        self.run_sync(move |backend| backend.fetch(key))
    }

    fn get_genesis_hash(&self) -> Hash {
        self.backend.get_genesis_hash()
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
            Err(_err) => panic!("RemoteNodeWithExecutor: sender was dropped"),
        })
        .wait()
}
