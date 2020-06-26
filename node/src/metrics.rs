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

use futures::StreamExt;
use std::convert::TryFrom;
use std::future::Future;

use sc_client::{light::blockchain::AuxStore, BlockImportNotification, BlockchainEvents as _};
use sc_service::{AbstractService, Error};
use sp_runtime::{generic::BlockId, traits::Block as _};
use substrate_prometheus_endpoint::prometheus::core::Atomic;
use substrate_prometheus_endpoint::{Gauge, Registry, U64};

use crate::pow::Difficulty;

pub fn register_metrics<S>(service: &S) -> Result<(), Error>
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
