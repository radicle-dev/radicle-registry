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
use sc_service::GenericChainSpec;
use sc_telemetry::TelemetryEndpoints;
use sp_core::{crypto::CryptoType, Pair};
use std::convert::TryFrom;

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = GenericChainSpec<GenesisConfig>;

const FFNET_GENESIS_RUNTIME_WASM: &[u8] = include_bytes!("../../runtime/ffnet_genesis.wasm");
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
    pub fn spec(&self, enable_telemetry: bool) -> ChainSpec {
        match self {
            Chain::Dev => dev(),
            Chain::LocalDevnet => local_devnet(),
            Chain::Devnet => devnet(),
            Chain::Ffnet => ffnet(enable_telemetry),
        }
    }
}

fn dev() -> ChainSpec {
    GenericChainSpec::from_genesis(
        "Radicle Registry isolated development",
        "dev",
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

fn ffnet(enable_telemetry: bool) -> ChainSpec {
    GenericChainSpec::from_genesis(
        "Radicle Registry ffnet",
        "ffnet",
        ffnet_genesis_config,
        // Addresses are defined here: https://github.com/radicle-dev/infra/tree/master/registry/ffnet
        vec![
            "/dns4/boot-0.ff.radicle.network./tcp/30333/p2p/QmdEvLkAS8mxETQy1RCbdmcPPzxSs9RbExFcWvwJZDXxjG"
                .parse().unwrap(),
            "/dns4/boot-1.ff.radicle.network./tcp/30333/p2p/QmceS5WYfDyKNtnzrxCw4TEL9nokvJkRi941oUzBvErsuD"
                .parse().unwrap()
        ],
        polkadot_telemetry(enable_telemetry),
        Some("ffnet"), // protocol_id
        Some(sc_service::Properties::try_from(PowAlgConfig::Blake3).unwrap()),
        None, // no extensions
    )
}

fn ffnet_genesis_config() -> GenesisConfig {
    use sp_core::crypto::Ss58Codec;

    // Public keys for Github handles
    let nunoalexandre =
        AccountId::from_ss58check("5CbJnuDccgarfidUapCAZZgx7rPWv6J5G4NK2H7Zcykh3EF8").unwrap();
    let codesandwich =
        AccountId::from_ss58check("5Caqj67GfbVRUBpyUpHBG6mQFzP6L9MjibkcrvriUWVLYfjf").unwrap();
    let geigerzaehler =
        AccountId::from_ss58check("5Chs9EgWihAkawHzszEnv3X3uBZMUmi98rUsbu2Fyw5gKHx6").unwrap();

    let init_balance = 1_000_000_000_000;
    let balances = vec![
        (codesandwich, init_balance),
        (geigerzaehler, init_balance),
        (nunoalexandre, init_balance),
    ];

    let sudo_key = geigerzaehler;

    GenesisConfig {
        system: Some(SystemConfig {
            code: FFNET_GENESIS_RUNTIME_WASM.to_vec(),
            changes_trie_config: Default::default(),
        }),
        pallet_balances: Some(BalancesConfig { balances }),
        pallet_sudo: Some(SudoConfig { key: sudo_key }),
    }
}

fn local_devnet() -> ChainSpec {
    GenericChainSpec::from_genesis(
        "Radicle Registry local devnet",
        "local-devnet",
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

fn polkadot_telemetry(enable: bool) -> Option<TelemetryEndpoints> {
    if enable {
        Some(
            TelemetryEndpoints::new(vec![("wss://telemetry.polkadot.io/submit/".to_string(), 0)])
                .unwrap(),
        )
    } else {
        None
    }
}
