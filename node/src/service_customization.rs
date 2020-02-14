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

//! This module contains chunks of code needed for customizations in [crate::service] module.

/// The new_full node import building function body
macro_rules! new_full {
    ($config:expr) => {{
        let (builder, import_setup, inherent_data_providers) = new_full_start!($config);
        let block_import = import_setup.expect("No import setup set for miner");

        let service = builder
            .with_network_protocol(|_| Ok(NodeProtocol::new()))?
            .build()?;

        let proposer = sc_basic_authorship::ProposerFactory {
            client: service.client(),
            transaction_pool: service.transaction_pool(),
        };

        sc_consensus_pow::start_mine(
            block_import,
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
    }};
}

/// The node with_import_queue closure body
macro_rules! node_import_queue {
    ($client:expr, $select_chain:expr, $inherent_data_providers:expr) => {{
        let pow_block_import = sc_consensus_pow::PowBlockImport::new(
            $client.clone(),
            $client,
            crate::dummy_pow::DummyPow,
            0,
            $select_chain,
            $inherent_data_providers,
        );
        let block_import = Box::new(pow_block_import);
        let import_queue = sc_consensus_pow::import_queue(
            block_import.clone(),
            crate::dummy_pow::DummyPow,
            $inherent_data_providers,
        )?;
        (block_import, import_queue)
    }};
}
