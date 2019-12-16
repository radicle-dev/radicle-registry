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
//! * [local_devnet] for runnning a cluster of three nodes locally using Aura consensus.
use aura_primitives::sr25519::AuthorityId as AuraId;
use primitives::{Pair, Public};
use radicle_registry_runtime::{
    AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig, SudoConfig, SystemConfig,
    WASM_BINARY,
};
use sc_finality_grandpa::AuthorityId as GrandpaId;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::ChainSpec<GenesisConfig>;

pub fn from_id(id: &str) -> Option<ChainSpec> {
    if id == "dev" {
        Some(dev())
    } else if id == "local-devnet" {
        Some(local_devnet())
    } else {
        None
    }
}

pub fn dev() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        dev_genesis_config,
        vec![], // boot nodes
        None,   // telemetry endpoints
        // protocol_id
        Some("dev"),
        None, // no properties
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
    let aura_authorities = vec![get_from_seed::<AuraId>("Alice")];
    let grandpa_authorities = vec![(get_from_seed::<GrandpaId>("Alice"), 1)];
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
            vesting: vec![],
        }),
        pallet_sudo: Some(SudoConfig { key: root_key }),
        pallet_aura: Some(AuraConfig {
            authorities: aura_authorities,
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: grandpa_authorities,
        }),
    }
}

pub fn local_devnet() -> ChainSpec {
    ChainSpec::from_genesis(
        "local devnet",
        "local-devnet",
        local_dev_genesis_config,
        vec![], // boot nodes
        None,   // telemetry endpoints
        // protocol_id
        Some("local-devnet"),
        None, // no properties
        None, // no extensions
    )
}

fn local_dev_genesis_config() -> GenesisConfig {
    let endowed_accounts = vec![
        get_from_seed::<AccountId>("Alice"),
        get_from_seed::<AccountId>("Bob"),
        get_from_seed::<AccountId>("Alice//stash"),
        get_from_seed::<AccountId>("Bob//stash"),
    ];
    let aura_authorities = vec![
        get_from_seed::<AuraId>("Alice"),
        get_from_seed::<AuraId>("Bob"),
        get_from_seed::<AuraId>("Charlie"),
    ];
    let grandpa_authorities = vec![
        (get_from_seed::<GrandpaId>("Alice"), 1),
        (get_from_seed::<GrandpaId>("Bob"), 1),
        (get_from_seed::<GrandpaId>("Charlie"), 1),
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
            vesting: vec![],
        }),
        pallet_sudo: Some(SudoConfig { key: root_key }),
        pallet_aura: Some(AuraConfig {
            authorities: aura_authorities,
        }),
        pallet_grandpa: Some(GrandpaConfig {
            authorities: grandpa_authorities,
        }),
    }
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}
