use aura_primitives::sr25519::AuthorityId as AuraId;
use primitives::{Pair, Public};
use radicle_registry_runtime::{
    AccountId, AuraConfig, BalancesConfig, GenesisConfig, SudoConfig, SystemConfig, WASM_BINARY,
};

/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = substrate_service::ChainSpec<GenesisConfig>;

pub fn dev() -> ChainSpec {
    ChainSpec::from_genesis(
        "Development",
        "dev",
        dev_genesis_config,
        vec![],
        None, // boot nodes
        None, // telemetry endpoints
        None, // protocol_id
        None, // no extensions
    )
}

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

fn dev_genesis_config() -> GenesisConfig {
    let endowed_accounts = vec![
        get_from_seed::<AccountId>("Alice"),
        get_from_seed::<AccountId>("Bob"),
        get_from_seed::<AccountId>("Alice//stash"),
        get_from_seed::<AccountId>("Bob//stash"),
    ];
    let authorities = vec![get_from_seed::<AuraId>("Alice")];
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
        pallet_aura: Some(AuraConfig { authorities }),
    }
}
