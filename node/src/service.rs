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

//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.
//!
//! **To all future Radicle developers:**
//!
//! This file is a hand-modified version of Substrate template.
//! It's located in Substrate repository under `bin/node-template/node/src/service.rs`.
//! Current version of this file is based on commit `09abd3b436ea568a47ec4fc47f933728d8cf466b`.
//! The changes should be easy to replay by following the [TEMPLATE DIFF] comments or diffing.
//! Bigger replacement blocks of code are stored in [crate::service_customization] module.

// [TEMPLATE DIFF] The original file contains parts that trigger clippy, we don't want to fix them
#![allow(clippy::all)]

use std::sync::Arc;
use sc_client::LongestChain;
// [TEMPLATE DIFF] The types are changed from node_template_runtime to Radicle custom
use radicle_registry_runtime::{opaque::Block, RuntimeApi};
use sc_service::{error::{Error as ServiceError}, AbstractService, Configuration, ServiceBuilder};
use sp_inherents::InherentDataProviders;
use sc_executor::native_executor_instance;


// Our native executor instance.
native_executor_instance!(
        pub Executor,
        // [TEMPLATE DIFF] The type is changed to Radicle custom
        radicle_registry_runtime::api::dispatch,
        // [TEMPLATE DIFF] The type is changed to Radicle custom
        radicle_registry_runtime::native_version,
);

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
    ($config:expr) => {{
        let mut import_setup = None;
        let inherent_data_providers = sp_inherents::InherentDataProviders::new();

        // [TEMPLATE DIFF] All the types are changed to Radicle custom
        let builder = sc_service::ServiceBuilder::new_full::<
            radicle_registry_runtime::opaque::Block,
            radicle_registry_runtime::RuntimeApi,
            crate::service::Executor,
        >($config)?
        .with_select_chain(|_config, backend| {
            Ok(sc_client::LongestChain::new(backend.clone()))
        })?
        .with_transaction_pool(|config, client, _fetcher| {
            let pool_api = sc_transaction_pool::FullChainApi::new(client.clone());
            let pool = sc_transaction_pool::BasicPool::new(config, std::sync::Arc::new(pool_api));
            Ok(pool)
        })?
        .with_import_queue(|config, client, select_chain, _transaction_pool| {
            // [TEMPLATE DIFF] The whole closure is replaced
            let (block_import, import_queue) =
                node_import_queue!(config, client, select_chain, inherent_data_providers.clone());
            import_setup = Some(block_import);
            Ok(import_queue)
        })?;

        (builder, import_setup, inherent_data_providers)
    }}
}

/// Builds a new service for a full client.
pub fn new_full(config: Configuration)
    -> Result<impl AbstractService, ServiceError> {
    // [TEMPLATE DIFF] The whole function is replaced
    new_full!(config)
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration)
    -> Result<impl AbstractService, ServiceError>
{
    let inherent_data_providers = InherentDataProviders::new();

    ServiceBuilder::new_light::<Block, RuntimeApi, Executor>(config)?
        .with_select_chain(|_config, backend| {
            Ok(LongestChain::new(backend.clone()))
        })?
        .with_transaction_pool(|config, client, fetcher| {
            let fetcher = fetcher
                .ok_or_else(|| "Trying to start light transaction pool without active fetcher")?;

            let pool_api = sc_transaction_pool::LightChainApi::new(client.clone(), fetcher.clone());
            let pool = sc_transaction_pool::BasicPool::with_revalidation_type(
                config, Arc::new(pool_api), sc_transaction_pool::RevalidationType::Light,
            );
            Ok(pool)
        })?
        // [TEMPLATE DIFF] Change FPRB queue to regular one
        .with_import_queue(|config, client, select_chain, _transaction_pool| {
                // [TEMPLATE DIFF] The whole closure is replaced
                let (_, import_queue) =
                    node_import_queue!(config, client, select_chain, inherent_data_providers.clone());
                Ok(import_queue)
            }
        )?
        // [TEMPLATE DIFF] The finality proof provider is completely removed
        .build()
}
