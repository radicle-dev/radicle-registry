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

use std::sync::Arc;

use sc_client::LongestChain;
use sc_executor::native_executor_instance;
pub use sc_executor::NativeExecutor;
use sc_finality_grandpa::{self, FinalityProofProvider as GrandpaFinalityProofProvider};
use sc_network::construct_simple_protocol;
use sc_service::{error::Error as ServiceError, AbstractService, Configuration, ServiceBuilder};
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;
use sp_inherents::InherentDataProviders;

use radicle_registry_runtime::{self, opaque::Block, GenesisConfig, RuntimeApi};

// Our native executor instance.
native_executor_instance!(
        pub Executor,
        radicle_registry_runtime::api::dispatch,
        radicle_registry_runtime::native_version,
);

construct_simple_protocol! {
    /// Demo protocol attachment for substrate.
    pub struct NodeProtocol where Block = Block { }
}

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
    ($config:expr) => {{
        let inherent_data_providers = sp_inherents::InherentDataProviders::new();

        let builder = sc_service::ServiceBuilder::new_full::<
            radicle_registry_runtime::opaque::Block,
            radicle_registry_runtime::RuntimeApi,
            crate::service::Executor,
        >($config)?
        .with_select_chain(|_config, backend| Ok(sc_client::LongestChain::new(backend.clone())))?
        .with_transaction_pool(|config, client, _fetcher| {
            let pool_api = sc_transaction_pool::FullChainApi::new(client);
            let pool = sc_transaction_pool::BasicPool::new(config, std::sync::Arc::new(pool_api));
            Ok(pool)
        })?
        .with_import_queue(|_config, client, select_chain, _transaction_pool| {
            let block_import = pow_consensus::PowBlockImport::new(
                client.clone(),
                client,
                crate::dummy_pow::DummyPow,
                0,
                select_chain,
                inherent_data_providers.clone(),
            );
            let import_queue = pow_consensus::import_queue(
                Box::new(block_import),
                crate::dummy_pow::DummyPow,
                inherent_data_providers.clone(),
            )?;
            Ok(import_queue)
        })?;
        (builder, inherent_data_providers)
    }};
}

/// Builds a new service for a full client.
pub fn new_full(
    config: Configuration<GenesisConfig>,
) -> Result<impl AbstractService, ServiceError> {
    let (builder, inherent_data_providers) = new_full_start!(config);

    let service = builder
        .with_network_protocol(|_| Ok(NodeProtocol::new()))?
        .with_finality_proof_provider(|client, backend| {
            Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, client)) as _)
        })?
        .build()?;

    let proposer = sc_basic_authorship::ProposerFactory {
        client: service.client(),
        transaction_pool: service.transaction_pool(),
    };

    pow_consensus::start_mine(
        Box::new(service.client()),
        service.client(),
        crate::dummy_pow::DummyPow,
        proposer,
        None,
        0,
        service.network(),
        std::time::Duration::new(2, 0),
        service.select_chain(),
        inherent_data_providers,
        sp_consensus::AlwaysCanAuthor,
    );

    Ok(service)
}

/// Builds a new service for a light client.
pub fn new_light(
    config: Configuration<GenesisConfig>,
) -> Result<impl AbstractService, ServiceError> {
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
        .with_import_queue_and_fprb(
            |_config, client, backend, fetcher, _select_chain, _tx_pool| {
                let fetch_checker = fetcher
                    .map(|fetcher| fetcher.checker().clone())
                    .ok_or_else(|| {
                        "Trying to start light import queue without active fetch checker"
                    })?;
                let grandpa_block_import =
                    sc_finality_grandpa::light_block_import::<_, _, _, RuntimeApi>(
                        client.clone(),
                        backend,
                        &*client.clone(),
                        Arc::new(fetch_checker),
                    )?;
                let finality_proof_import = grandpa_block_import.clone();
                let finality_proof_request_builder =
                    finality_proof_import.create_finality_proof_request_builder();

                let import_queue = sc_consensus_aura::import_queue::<_, _, _, AuraPair, ()>(
                    sc_consensus_aura::SlotDuration::get_or_compute(&*client)?,
                    grandpa_block_import,
                    None,
                    Some(Box::new(finality_proof_import)),
                    client,
                    inherent_data_providers.clone(),
                    None,
                )?;

                Ok((import_queue, finality_proof_request_builder))
            },
        )?
        .with_network_protocol(|_| Ok(NodeProtocol::new()))?
        .with_finality_proof_provider(|client, backend| {
            Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, client)) as _)
        })?
        .build()
}
