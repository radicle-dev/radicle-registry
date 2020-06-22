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

//! Provides constructor functions to create [ChainSpec]s.
use crate::pow::config::Config as PowAlgConfig;
use radicle_registry_runtime::{
    AccountId, Balance, BalancesConfig, GenesisConfig, SudoConfig, SystemConfig,
};
use sc_service::{config::MultiaddrWithPeerId, ChainType, GenericChainSpec};
use sp_core::{crypto::CryptoType, Pair};
use std::convert::TryFrom;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = GenericChainSpec<GenesisConfig>;

const FFNET_CHAIN_SPEC: &[u8] = include_bytes!("./chain_spec/ffnet.json");
const LATEST_RUNTIME_WASM: &[u8] = include_bytes!("../../runtime/latest.wasm");

/// Chain for local development with a single node.
///
/// If `runtime` is given, it is used as the genesis runtime. Uses dummy PoW that does not eat up
/// your CPU.
pub fn dev(runtime: Option<Vec<u8>>) -> ChainSpec {
    ChainParams {
        id: String::from("dev"),
        chain_type: ChainType::Development,
        boot_nodes: vec![],
        pow_alg: PowAlgConfig::Dummy,
        runtime: runtime.unwrap_or_else(|| LATEST_RUNTIME_WASM.to_owned()),
        balances: dev_balances(),
        sudo_key: account_id("Alice"),
    }
    .into_chain_spec()
}

/// Chain that is running on the cloud and is frequently updated and reset.
pub fn devnet() -> ChainSpec {
    ChainParams {
        id: String::from("devnet"),
        chain_type: ChainType::Development,
        boot_nodes: vec![
            // The peer ID is the public key generated from the secret key 0x000...001.
            "/ip4/35.233.120.254/tcp/30333/p2p/QmRpheLN4JWdAnY7HGJfWFNbfkQCb6tFf4vvA6hgjMZKrR"
                .parse()
                .expect("Parsing a genesis peer address failed"),
        ],
        pow_alg: PowAlgConfig::Blake3,
        runtime: LATEST_RUNTIME_WASM.to_owned(),
        balances: dev_balances(),
        sudo_key: account_id("Alice"),
    }
    .into_chain_spec()
}

/// Chain for running a cluster of nodes locally.
///
/// If `runtime` is given, it is used as the genesis runtime. Similar to [dev] but uses proper PoW
/// consensus.
pub fn local_devnet(runtime: Option<Vec<u8>>) -> ChainSpec {
    ChainParams {
        id: String::from("local-devnet"),
        chain_type: ChainType::Development,
        boot_nodes: vec![],
        pow_alg: PowAlgConfig::Blake3,
        runtime: runtime.unwrap_or_else(|| LATEST_RUNTIME_WASM.to_owned()),
        balances: dev_balances(),
        sudo_key: account_id("Alice"),
    }
    .into_chain_spec()
}

/// First public test net.
pub fn ffnet() -> ChainSpec {
    ChainSpec::from_json_bytes(FFNET_CHAIN_SPEC).expect("Unable to parse ffnet chain spec")
}

/// Parameters to construct a [ChainSpec] with [ChainParams::into_chain_spec].
#[derive(Debug, Clone)]
struct ChainParams {
    id: String,
    chain_type: ChainType,
    boot_nodes: Vec<MultiaddrWithPeerId>,
    pow_alg: PowAlgConfig,
    runtime: Vec<u8>,
    balances: Vec<(AccountId, Balance)>,
    sudo_key: AccountId,
}

impl ChainParams {
    fn into_chain_spec(self) -> ChainSpec {
        let ChainParams {
            id,
            chain_type,
            boot_nodes,
            pow_alg,
            runtime,
            balances,
            sudo_key,
        } = self;
        let make_genesis_config = move || GenesisConfig {
            system: Some(SystemConfig {
                code: runtime.clone(),
                changes_trie_config: Default::default(),
            }),
            pallet_balances: Some(BalancesConfig {
                balances: balances.clone(),
            }),
            pallet_sudo: Some(SudoConfig { key: sudo_key }),
        };
        GenericChainSpec::from_genesis(
            &id,
            &id,
            chain_type,
            make_genesis_config,
            boot_nodes,
            None, // telemetry endpoints
            Some(&id),
            Some(sc_service::Properties::try_from(pow_alg).unwrap()),
            None, // no extensions
        )
    }
}

fn dev_balances() -> Vec<(AccountId, Balance)> {
    let init_balance = 1u128 << 60;
    vec![
        (account_id("Alice"), init_balance),
        (account_id("Bob"), init_balance),
        (account_id("Alice//stash"), init_balance),
        (account_id("Bob//stash"), init_balance),
    ]
}

/// Helper function to generate an account ID from a seed
fn account_id(seed: &str) -> AccountId {
    <AccountId as CryptoType>::Pair::from_string(&format!("//{}", seed), None)
        .expect("Parsing the account key pair seed failed")
        .public()
}
