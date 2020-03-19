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

//! Functions to create [sc_service::Service] values used for commands.
//!
//! This module is based on `service` module from the Substrate node template.

use std::convert::TryFrom;
use std::sync::Arc;

use sc_client::LongestChain;
use sc_executor::native_executor_instance;
use sc_service::{AbstractService, Configuration, Error as ServiceError, ServiceBuilder};
use sp_inherents::InherentDataProviders;

use crate::pow::config::Config as PowAlgConfig;
use radicle_registry_runtime::{opaque::Block, RuntimeApi};

// Our native executor instance.
native_executor_instance!(
        pub Executor,
        radicle_registry_runtime::api::dispatch,
        radicle_registry_runtime::native_version,
);

/// Starts a `ServiceBuilder` for a full service.
macro_rules! new_full_start {
    ($config:expr) => {{
        let mut import_setup = None;
        let inherent_data_providers = sp_inherents::InherentDataProviders::new();

        let builder = sc_service::ServiceBuilder::new_full::<Block, RuntimeApi, Executor>($config)?
            .with_select_chain(|_config, backend| {
                Ok(sc_client::LongestChain::new(backend.clone()))
            })?
            .with_transaction_pool(|config, client, _fetcher| {
                let pool_api = sc_transaction_pool::FullChainApi::new(client);
                let pool =
                    sc_transaction_pool::BasicPool::new(config, std::sync::Arc::new(pool_api));
                Ok(pool)
            })?
            .with_import_queue(|config, client, select_chain, _transaction_pool| {
                let (block_import, import_queue) = node_import_queue!(
                    config,
                    client,
                    select_chain,
                    inherent_data_providers.clone()
                );
                import_setup = Some(block_import);
                Ok(import_queue)
            })?;

        (builder, import_setup, inherent_data_providers)
    }};
}

/// Start mining on full node
macro_rules! start_mine {
    ($block_import:expr, $service:expr, $proposer:expr, $inherent_data_providers:expr, $pow_alg:expr) => {{
        sc_consensus_pow::start_mine(
            $block_import,
            $service.client(),
            $pow_alg,
            $proposer,
            None,
            0,
            $service.network(),
            std::time::Duration::new(2, 0),
            $service.select_chain(),
            $inherent_data_providers,
            sp_consensus::AlwaysCanAuthor,
        );
    }};
}

/// The node with_import_queue closure body
macro_rules! node_import_queue {
    ($config:expr, $client:expr, $select_chain:expr, $inherent_data_providers:expr) => {{
        match PowAlgConfig::try_from($config)? {
            PowAlgConfig::Dummy => node_import_queue_for_pow_alg!(
                $client,
                $select_chain,
                $inherent_data_providers,
                crate::pow::dummy_pow::DummyPow
            ),
            PowAlgConfig::Blake3 => node_import_queue_for_pow_alg!(
                $client,
                $select_chain,
                $inherent_data_providers,
                crate::pow::blake3_pow::Blake3Pow::new()
            ),
        }
    }};
}

/// The node with_import_queue closure body when PoW algorithm is known
macro_rules! node_import_queue_for_pow_alg {
    ($client:expr, $select_chain:expr, $inherent_data_providers:expr, $pow_alg:expr) => {{
        let pow_block_import = sc_consensus_pow::PowBlockImport::new(
            $client.clone(),
            $client,
            $pow_alg,
            0,
            $select_chain,
            $inherent_data_providers,
        );
        let block_import_box = Box::new(pow_block_import);
        let import_queue = sc_consensus_pow::import_queue(
            block_import_box.clone(),
            $pow_alg,
            $inherent_data_providers,
        )?;
        let block_import = block_import_box as sp_consensus::import_queue::BoxBlockImport<_, _>;
        (block_import, import_queue)
    }};
}

/// Builds a new service for a full client and starts the PoW miner.
pub fn new_full(config: Configuration) -> Result<impl AbstractService, ServiceError> {
    let pow_alg = PowAlgConfig::try_from(&config)?;
    let (builder, import_setup, inherent_data_providers) = new_full_start!(config);
    let block_import = import_setup.expect("No import setup set for miner");

    let service = builder.build()?;

    let proposer =
        sc_basic_authorship::ProposerFactory::new(service.client(), service.transaction_pool());

    match pow_alg {
        PowAlgConfig::Dummy => start_mine!(
            block_import,
            service,
            proposer,
            inherent_data_providers,
            crate::pow::dummy_pow::DummyPow
        ),
        PowAlgConfig::Blake3 => start_mine!(
            block_import,
            service,
            proposer,
            inherent_data_providers,
            crate::pow::blake3_pow::Blake3Pow::new()
        ),
    }
    Ok(service)
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration) -> Result<impl AbstractService, ServiceError> {
    let inherent_data_providers = InherentDataProviders::new();

    ServiceBuilder::new_light::<Block, RuntimeApi, Executor>(config)?
        .with_select_chain(|_config, backend| Ok(LongestChain::new(backend.clone())))?
        .with_transaction_pool(|config, client, fetcher| {
            let fetcher = fetcher
                .ok_or_else(|| "Trying to start light transaction pool without active fetcher")?;

            let pool_api = sc_transaction_pool::LightChainApi::new(client, fetcher);
            let pool = sc_transaction_pool::BasicPool::with_revalidation_type(
                config,
                Arc::new(pool_api),
                sc_transaction_pool::RevalidationType::Light,
            );
            Ok(pool)
        })?
        .with_import_queue(|config, client, select_chain, _transaction_pool| {
            let (_, import_queue) = node_import_queue!(
                config,
                client,
                select_chain,
                inherent_data_providers.clone()
            );
            Ok(import_queue)
        })?
        .build()
}

/// Build a new service to be used for one-shot commands.
pub fn new_for_command(
    config: Configuration,
) -> Result<impl sc_service::ServiceBuilderCommand<Block = Block>, ServiceError> {
    Ok(new_full_start!(config).0)
}
