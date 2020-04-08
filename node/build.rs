use vergen::{generate_cargo_keys, ConstantsFlags};

const GENESIS_CHAIN_ENV: &str = "GENESIS_CHAIN";
const GENESIS_CHAIN_CFG: &str = "genesis_chain";

fn main() {
    generate_cargo_keys(ConstantsFlags::SHA_SHORT).unwrap();
    set_genesis_chain_cfg();
}

fn set_genesis_chain_cfg() {
    if let Ok(genesis_chain) = std::env::var(GENESIS_CHAIN_ENV) {
        println!(
            "cargo:rustc-cfg={}=\"{}\"",
            GENESIS_CHAIN_CFG, genesis_chain
        );
    }
    println!("cargo:rerun-if-env-changed={}", GENESIS_CHAIN_ENV);
}
