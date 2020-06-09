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

//! Provides [Chain] and [ChainSpec]s for various chains we want to run.
//!
//! Available chain specs
//! * [dev] for runnning a single node locally and develop against it.
//! * [local_devnet] for runnning a cluster of three nodes locally.
use crate::pow::config::Config as PowAlgConfig;
use radicle_registry_runtime::{
    AccountId, BalancesConfig, GenesisConfig, SudoConfig, SystemConfig,
};
use sc_service::{ChainType, GenericChainSpec};
use sp_core::{crypto::CryptoType, Pair};
use std::convert::TryFrom;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = GenericChainSpec<GenesisConfig>;

const FFNET_CHAIN_SPEC: &[u8] = include_bytes!("./chain_spec/ffnet.json");
const LATEST_RUNTIME_WASM: &[u8] = include_bytes!("../../runtime/latest.wasm");

/// Possible chains.
///
/// Use [Chain::spec] to get the corresponding [ChainSpec].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Chain {
    Dev,
    LocalDevnet,
    Devnet,
    Ffnet,
}

impl Chain {
    pub fn spec(&self) -> ChainSpec {
        match self {
            Chain::Dev => dev(),
            Chain::LocalDevnet => local_devnet(),
            Chain::Devnet => devnet(),
            Chain::Ffnet => ffnet(),
        }
    }
}

fn dev() -> ChainSpec {
    GenericChainSpec::from_genesis(
        "Radicle Registry isolated development",
        "dev",
        ChainType::Development,
        dev_genesis_config,
        vec![],      // boot nodes
        None,        // telemetry endpoints
        Some("dev"), // protocol_id
        Some(sc_service::Properties::try_from(PowAlgConfig::Dummy).unwrap()),
        None, // no extensions
    )
}

fn devnet() -> ChainSpec {
    GenericChainSpec::from_genesis(
        "Radicle Registry devnet",
        "devnet",
        ChainType::Custom("devnet".to_string()),
        dev_genesis_config,
        // boot nodes
        // From key 000...001
        vec![
            "/ip4/35.233.120.254/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR"
                .parse()
                .expect("Parsing a genesis peer address failed"),
        ],
        None,           // telemetry endpoints
        Some("devnet"), // protocol_id
        Some(sc_service::Properties::try_from(PowAlgConfig::Blake3).unwrap()),
        None, // no extensions
    )
}

fn ffnet() -> ChainSpec {
    ChainSpec::from_json_bytes(FFNET_CHAIN_SPEC).expect("Unable to parse ffnet chain spec")
}

fn local_devnet() -> ChainSpec {
    GenericChainSpec::from_genesis(
        "Radicle Registry local devnet",
        "local-devnet",
        ChainType::Development,
        dev_genesis_config,
        vec![], // boot nodes
        None,   // telemetry endpoints
        // protocol_id
        Some("local-devnet"),
        Some(sc_service::Properties::try_from(PowAlgConfig::Blake3).unwrap()),
        None, // no extensions
    )
}

fn dev_genesis_config() -> GenesisConfig {
    let init_balance = 1u128 << 60;
    let balances = vec![
        (account_id("Alice"), init_balance),
        (account_id("Bob"), init_balance),
        (account_id("Alice//stash"), init_balance),
        (account_id("Bob//stash"), init_balance),
    ];
    let sudo_key = account_id("Alice");
    GenesisConfig {
        system: Some(SystemConfig {
            code: LATEST_RUNTIME_WASM.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig { balances }),
        pallet_sudo: Some(SudoConfig { key: sudo_key }),
    }
}

/// Helper function to generate an account ID from a seed
fn account_id(seed: &str) -> AccountId {
    <AccountId as CryptoType>::Pair::from_string(&format!("//{}", seed), None)
        .expect("Parsing the account key pair seed failed")
        .public()
}
