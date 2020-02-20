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

//! Defines the [ChainSpec]s for various chains we want to run.
//!
//! Available chain specs
//! * [dev] for runnning a single node locally and develop against it.
//! * [local_devnet] for runnning a cluster of three nodes locally.
use crate::pow_alg_config::PowAlgConfig;
use radicle_registry_runtime::{
    AccountId, BalancesConfig, GenesisConfig, SudoConfig, SystemConfig, WASM_BINARY,
};
use sp_core::{Pair, Public};
use std::convert::TryFrom;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::ChainSpec<GenesisConfig>;

pub fn load_spec(id: &str) -> Result<Option<ChainSpec>, String> {
    match from_id(id) {
        Some(spec) => Ok(Some(spec)),
        _ => Err(format!(
            "Unknown chain spec \"{}\". You must run the node with --dev",
            id
        )),
    }
}

pub fn from_id(id: &str) -> Option<ChainSpec> {
    if id == "dev" {
        Some(dev())
    } else if id == "devnet" {
        Some(devnet())
    } else if id == "local-devnet" {
        Some(local_devnet())
    } else {
        None
    }
}

pub fn dev() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development, isolated node",
        "dev",
        dev_genesis_config,
        vec![], // boot nodes
        None,   // telemetry endpoints
        // protocol_id
        Some("dev"),
        Some(sc_service::Properties::try_from(PowAlgConfig::Dummy).unwrap()),
        None, // no extensions
    )
}

fn dev_genesis_config() -> GenesisConfig {
    let endowed_accounts = vec![
        get_from_seed::<AccountId>("Alice"),
        get_from_seed::<AccountId>("Bob"),
        get_from_seed::<AccountId>("Alice//stash"),
        get_from_seed::<AccountId>("Bob//stash"),
    ];
    let root_key = get_from_seed::<AccountId>("Alice");
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

pub fn devnet() -> ChainSpec {
    ChainSpec::from_genesis(
        "devnet",
        "devnet",
        devnet_genesis_config,
        // boot nodes
        // From key 000...001
        vec![
            "/ip4/35.233.120.254/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR"
                .to_string(),
        ],
        None, // telemetry endpoints
        // protocol_id
        Some("devnet"),
        Some(sc_service::Properties::try_from(PowAlgConfig::Dummy).unwrap()),
        None, // no extensions
    )
}

pub fn local_devnet() -> ChainSpec {
    ChainSpec::from_genesis(
        "local devnet, isolated on one machine",
        "local-devnet",
        devnet_genesis_config,
        vec![], // boot nodes
        None,   // telemetry endpoints
        // protocol_id
        Some("local-devnet"),
        Some(sc_service::Properties::try_from(PowAlgConfig::Dummy).unwrap()),
        None, // no extensions
    )
}

fn devnet_genesis_config() -> GenesisConfig {
    let endowed_accounts = vec![
        get_from_seed::<AccountId>("Alice"),
        get_from_seed::<AccountId>("Bob"),
        get_from_seed::<AccountId>("Alice//stash"),
        get_from_seed::<AccountId>("Bob//stash"),
    ];
    let root_key = get_from_seed::<AccountId>("Alice");
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

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}
