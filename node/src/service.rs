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

use futures::StreamExt;
use std::convert::TryFrom;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

// TODO remove in favor of substrate_prometheus_endpoint::prometheus after substrate upgrade
use prometheus::core::Atomic;
use sc_client::BlockImportNotification;
use sc_client::{light::blockchain::AuxStore, BlockchainEvents as _, LongestChain};
use sc_executor::native_executor_instance;
use sc_service::{AbstractService, Configuration, Error, ServiceBuilder};
use sp_inherents::InherentDataProviders;
use sp_runtime::generic::BlockId;
use sp_runtime::traits::Block as _;
use substrate_prometheus_endpoint::{Gauge, Registry, U64};

use crate::pow::{blake3_pow::Blake3Pow, config::Config, dummy_pow::DummyPow, Difficulty};
use radicle_registry_runtime::{registry::AuthoringInherentData, AccountId, Block, RuntimeApi};

// Our native executor instance.
native_executor_instance!(
        pub Executor,
        radicle_registry_runtime::api::dispatch,
        radicle_registry_runtime::native_version,
);

/// Starts a `ServiceBuilder` for a full service.
macro_rules! new_full_start {
    ($config:expr, $inherent_data_providers: expr) => {{
        let mut import_setup = None;
        let builder = sc_service::ServiceBuilder::new_full::<Block, RuntimeApi, Executor>($config)?
            .with_select_chain(|_config, backend| {
                Ok(sc_client::LongestChain::new(backend.clone()))
            })?
            .with_transaction_pool(|options, client, _fetcher| {
                let pool_api = sc_transaction_pool::FullChainApi::new(client);
                let pool =
                    sc_transaction_pool::BasicPool::new(options, std::sync::Arc::new(pool_api));
                Ok(pool)
            })?
            .with_import_queue(|config, client, select_chain, _transaction_pool| {
                let (block_import, import_queue) = node_import_queue!(
                    config,
                    client,
                    select_chain,
                    $inherent_data_providers.clone()
                );
                import_setup = Some(block_import);
                Ok(import_queue)
            })?;

        (builder, import_setup)
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
            Duration::new(2, 0),
            $service.select_chain(),
            $inherent_data_providers,
            sp_consensus::AlwaysCanAuthor,
        );
    }};
}

/// The node with_import_queue closure body
macro_rules! node_import_queue {
    ($config:expr, $client:expr, $select_chain:expr, $inherent_data_providers:expr) => {{
        match Config::try_from($config)? {
            Config::Dummy => node_import_queue_for_pow_alg!(
                $client,
                $select_chain,
                $inherent_data_providers,
                DummyPow
            ),
            Config::Blake3 => node_import_queue_for_pow_alg!(
                $client,
                $select_chain,
                $inherent_data_providers,
                Blake3Pow::new($client.clone())
            ),
        }
    }};
}

/// The node with_import_queue closure body when PoW algorithm is known
macro_rules! node_import_queue_for_pow_alg {
    ($client:expr, $select_chain:expr, $inherent_data_providers:expr, $pow_alg:expr) => {{
        let pow_block_import = sc_consensus_pow::PowBlockImport::new(
            $client.clone(),
            $client.clone(),
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

/// Builds a new service for a full client.
///
/// Starts a miner if `opt_block_author` was provided.
pub fn new_full(
    config: Configuration,
    opt_block_author: Option<AccountId>,
) -> Result<impl AbstractService, Error> {
    log::info!(
        "Native runtime version: spec={} impl={}",
        radicle_registry_runtime::VERSION.spec_version,
        radicle_registry_runtime::VERSION.impl_version,
    );

    let pow_alg = Config::try_from(&config)?;
    let inherent_data_providers = InherentDataProviders::new();
    let (builder, import_setup) = new_full_start!(config, inherent_data_providers.clone());
    let block_import = import_setup.expect("No import setup set for miner");

    let service = builder.build()?;
    register_metrics(&service)?;

    if let Some(block_author) = opt_block_author {
        let client = service.client();
        service.spawn_essential_task(
            "mined-block-notifier",
            client.import_notification_stream().for_each(|info| {
                if info.origin == sp_consensus::BlockOrigin::Own {
                    log::info!("Imported own block #{} ({})", info.header.number, info.hash)
                }
                futures::future::ready(())
            }),
        );

        let authoring_inherent_data = AuthoringInherentData { block_author };

        // Can only fail if a provider with the same name is already registered.
        inherent_data_providers
            .register_provider(authoring_inherent_data)
            .unwrap();

        let proposer =
            sc_basic_authorship::ProposerFactory::new(service.client(), service.transaction_pool());

        log::info!("Starting block miner");

        match pow_alg {
            Config::Dummy => start_mine!(
                block_import,
                service,
                proposer,
                inherent_data_providers,
                DummyPow
            ),
            Config::Blake3 => start_mine!(
                block_import,
                service,
                proposer,
                inherent_data_providers,
                Blake3Pow::new(client)
            ),
        }
    } else {
        log::info!("Mining is disabled");
    }

    Ok(service)
}

/// Builds a new service for a light client.
pub fn new_light(config: Configuration) -> Result<impl AbstractService, Error> {
    let service = ServiceBuilder::new_light::<Block, RuntimeApi, Executor>(config)?
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
            let (_, import_queue) =
                node_import_queue!(config, client, select_chain, InherentDataProviders::new());
            Ok(import_queue)
        })?
        .build()?;
    register_metrics(&service)?;
    Ok(service)
}

fn register_metrics<S>(service: &S) -> Result<(), Error>
where
    S: AbstractService,
    sc_client::Client<S::Backend, S::CallExecutor, S::Block, S::RuntimeApi>: AuxStore,
{
    let registry = match service.prometheus_registry() {
        Some(registry) => registry,
        None => {
            log::warn!("Prometheus is disabled, some metrics won't be collected");
            return Ok(());
        }
    };
    register_best_block_metrics(service, &registry)?;
    Ok(())
}

fn register_best_block_metrics<S>(service: &S, registry: &Registry) -> Result<(), Error>
where
    S: AbstractService,
    sc_client::Client<S::Backend, S::CallExecutor, S::Block, S::RuntimeApi>: AuxStore,
{
    let update_difficulty_gauge = create_difficulty_gauge_updater(service, registry)?;
    let update_block_size_gauges = create_block_size_gauges_updater(service, registry)?;
    let update_reorganization_gauges = create_reorganization_gauges_updater(registry)?;
    let task = service
        .client()
        .import_notification_stream()
        .for_each(move |info| {
            if info.is_new_best {
                update_difficulty_gauge(&info);
                update_block_size_gauges(&info);
                update_reorganization_gauges(&info);
            }
            futures::future::ready(())
        });
    spawn_metric_task(service, "best_block", task);
    Ok(())
}

fn create_difficulty_gauge_updater<S>(
    service: &S,
    registry: &Registry,
) -> Result<impl Fn(&BlockImportNotification<S::Block>), Error>
where
    S: AbstractService,
    sc_client::Client<S::Backend, S::CallExecutor, S::Block, S::RuntimeApi>: AuxStore,
{
    let difficulty_gauge = register_gauge::<U64>(
        &registry,
        "best_block_difficulty",
        "The difficulty of the best block in the chain",
    )?;
    let client = service.client();
    let updater = move |info: &BlockImportNotification<S::Block>| {
        let difficulty_res =
            sc_consensus_pow::PowAux::<Difficulty>::read::<_, S::Block>(&*client, &info.hash);
        let difficulty = match difficulty_res {
            Ok(difficulty) => u64::try_from(difficulty.difficulty).unwrap_or(u64::MAX),
            Err(_) => return,
        };
        difficulty_gauge.set(difficulty);
    };
    Ok(updater)
}

fn create_block_size_gauges_updater<S: AbstractService>(
    service: &S,
    registry: &Registry,
) -> Result<impl Fn(&BlockImportNotification<S::Block>), Error> {
    let transactions_gauge = register_gauge::<U64>(
        &registry,
        "best_block_transactions",
        "Number of transactions in the best block in the chain",
    )?;
    let length_gauge = register_gauge::<U64>(
        &registry,
        "best_block_length",
        "Length in bytes of the best block in the chain",
    )?;
    let client = service.client();
    let updater = move |info: &BlockImportNotification<S::Block>| {
        let body = match client.body(&BlockId::hash(info.hash)) {
            Ok(Some(body)) => body,
            _ => return,
        };
        transactions_gauge.set(body.len() as u64);
        let encoded_block = S::Block::encode_from(&info.header, &body);
        length_gauge.set(encoded_block.len() as u64);
    };
    Ok(updater)
}

fn create_reorganization_gauges_updater<S: AbstractService>(
    registry: &Registry,
) -> Result<impl Fn(&BlockImportNotification<S::Block>), Error> {
    let reorg_length_gauge = register_gauge::<U64>(
        &registry,
        "best_block_reorganization_length",
        "Number of blocks rolled back to establish the best block in the chain",
    )?;
    let reorg_count_gauge = register_gauge::<U64>(
        &registry,
        "best_block_reorganization_count",
        "Number of best block reorganizations, which occurred in the chain",
    )?;
    let updater = move |info: &BlockImportNotification<S::Block>| {
        reorg_length_gauge.set(info.retracted.len() as u64);
        if !info.retracted.is_empty() {
            reorg_count_gauge.inc();
        }
    };
    Ok(updater)
}

fn register_gauge<P: Atomic + 'static>(
    registry: &Registry,
    gauge_name: &str,
    gauge_help: &str,
) -> Result<Gauge<P>, Error> {
    let gauge = Gauge::new(gauge_name, gauge_help)
        .map_err(|e| format!("failed to create metric gauge '{}': {}", gauge_name, e))?;
    substrate_prometheus_endpoint::register(gauge, &registry)
        .map_err(|e| format!("failed to register metric gauge '{}': {}", gauge_name, e).into())
}

fn spawn_metric_task(
    service: &impl AbstractService,
    name: &str,
    task: impl Future<Output = ()> + Send + 'static,
) {
    // TODO turn into passing a string after upgrade
    let task_name = Box::leak(format!("{}_metric_notifier", name).into_boxed_str());
    service.spawn_task(&*task_name, task);
}

/// Build a new service to be used for one-shot commands.
pub fn new_for_command(
    config: Configuration,
) -> Result<impl sc_service::ServiceBuilderCommand<Block = Block>, Error> {
    let inherent_data_providers = InherentDataProviders::new();
    Ok(new_full_start!(config, inherent_data_providers).0)
}
