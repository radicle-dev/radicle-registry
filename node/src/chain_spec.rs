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

//! Provides chain spec. See [chain_spec] for more details.

use crate::pow::config::Config as PowAlgConfig;
use radicle_registry_runtime::{
    AccountId, BalancesConfig, GenesisConfig, SudoConfig, SystemConfig,
};
use sc_service::GenericChainSpec;
use sp_core::{crypto::CryptoType, Pair};
use std::convert::TryFrom;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = GenericChainSpec<GenesisConfig>;

const WASM_BINARY: &[u8] = include_bytes!("../../runtime/genesis_runtime.wasm");

/// Provides chain specs for various chains we want to run.
///
/// Chain is picked automatically based on the value of the `cfg` flag `genesis_chain`:
/// * 'dev' for runnning a single node locally and develop against it
/// * 'local_devnet' for runnning a cluster of nodes locally
/// * 'devnet' for runnning a development network of nodes
/// * 'ffnet' for runnning a "friends and family" network of nodes
/// If none of the above values are set, the compilation will fail.
pub fn chain_spec() -> ChainSpec {
    cfg_if::cfg_if! {
        if #[cfg(genesis_chain="dev")] {
            GenericChainSpec::from_genesis(
                "Development, isolated node",
                "dev",
                genesis_config,
                vec![], // boot nodes
                None, // telemetry endpoints
                Some("dev"), // protocol_id
                Some(sc_service::Properties::try_from(PowAlgConfig::Dummy).unwrap()),
                None, // no extensions
            )
        } else if #[cfg(genesis_chain="local-devnet")] {
            GenericChainSpec::from_genesis(
                "local devnet, isolated on one machine",
                "local-devnet",
                genesis_config,
                vec![], // boot nodes
                None,   // telemetry endpoints
                Some("local-devnet"), // protocol_id
                Some(sc_service::Properties::try_from(PowAlgConfig::Blake3).unwrap()),
                None, // no extensions
            )
        } else if #[cfg(genesis_chain="devnet")] {
            let boot_node = // From key 000...001
                "/ip4/35.233.120.254/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR"
                    .parse()
                    .expect("Parsing a genesis peer address failed");
            GenericChainSpec::from_genesis(
                "devnet",
                "devnet",
                genesis_config,
                vec![boot_node], // boot nodes
                None, // telemetry endpoints
                Some("devnet"), // protocol_id
                Some(sc_service::Properties::try_from(PowAlgConfig::Blake3).unwrap()),
                None, // no extensions
            )
        } else if #[cfg(genesis_chain="ffnet")] {
            GenericChainSpec::from_genesis(
                "ffnet",
                "ffnet",
                genesis_config,
                vec![], // boot nodes
                None, // telemetry endpoints
                Some("ffnet"), // protocol_id
                Some(sc_service::Properties::try_from(PowAlgConfig::Blake3).unwrap()),
                None, // no extensions
            )
        } else {
            compile_error! {
                "Environment variable GENESIS_CHAIN must be set to: \
                'dev', 'local-devnet', 'devnet' or 'ffnet'"
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(any(
            genesis_chain="dev",
            genesis_chain="local-devnet",
            genesis_chain="devnet",
            genesis_chain="ffnet",
        ))] {

        fn genesis_config() -> GenesisConfig {
            let endowed_accounts = vec![
                account_pub_key("Alice"),
                account_pub_key("Bob"),
                account_pub_key("Alice//stash"),
                account_pub_key("Bob//stash"),
            ];
            let root_key = account_pub_key("Alice");
            GenesisConfig {
                system: Some(SystemConfig {
                    code: WASM_BINARY.to_vec(),
                    changes_trie_config: Default::default(),
                }),
                pallet_balances: Some(BalancesConfig {
                    balances: endowed_accounts
                        .iter()
                        .cloned()
                        .map(|k| (k, 1 << 60))
                        .collect(),
                }),
                pallet_sudo: Some(SudoConfig { key: root_key }),
            }
        }

    }
}

/// Helper function to generate a crypto pair from the seed
fn account_pub_key(seed: &str) -> <<AccountId as CryptoType>::Pair as Pair>::Public {
    <AccountId as CryptoType>::Pair::from_string(&format!("//{}", seed), None)
        .expect("Parsing the account key pair seed failed")
        .public()
}
