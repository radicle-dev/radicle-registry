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
use futures03::compat::Executor01CompatExt;
use futures03::future::BoxFuture;
use futures03::task::SpawnExt;
use std::sync::Arc;

use crate::backend;
use crate::interface::*;

/// Client backend that wraps [crate::backend::RemoteNode] but spawns all futures in
/// its own executor using [tokio::runtime::Runtime].
#[derive(Clone)]
pub struct RemoteNodeWithExecutor {
    backend: backend::RemoteNode,
    runtime: Arc<tokio::runtime::Runtime>,
}

impl RemoteNodeWithExecutor {
    pub async fn create(host: url::Host) -> Result<Self, Error> {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let backend = Executor01CompatExt::compat(runtime.executor())
            .spawn_with_handle(backend::RemoteNode::create(host))
            .unwrap()
            .await?;
        Ok(RemoteNodeWithExecutor {
            backend,
            runtime: Arc::new(runtime),
        })
    }
}

#[async_trait::async_trait]
impl backend::Backend for RemoteNodeWithExecutor {
    async fn submit(
        &self,
        xt: backend::UncheckedExtrinsic,
    ) -> Result<BoxFuture<'static, Result<backend::TransactionApplied, Error>>, Error> {
        let exec = Executor01CompatExt::compat(self.runtime.executor());
        let backend = self.backend.clone();
        let handle = exec
            .spawn_with_handle(async move { backend.submit(xt).await })
            .unwrap();
        let fut = handle.await?;
        Ok(Box::pin(exec.spawn_with_handle(fut).unwrap()))
    }

    async fn fetch(
        &self,
        key: &[u8],
        block_hash: Option<BlockHash>,
    ) -> Result<Option<Vec<u8>>, Error> {
        let backend = self.backend.clone();
        let key = Vec::from(key);
        let handle = Executor01CompatExt::compat(self.runtime.executor())
            .spawn_with_handle(async move { backend.fetch(&key, block_hash).await })
            .unwrap();
        handle.await
    }

    fn get_genesis_hash(&self) -> Hash {
        self.backend.get_genesis_hash()
    }
}
