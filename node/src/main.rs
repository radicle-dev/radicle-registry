//! Substrate Node Template CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;

pub use substrate_cli::{error, IntoExit, VersionInfo};

fn main() {
    let version = VersionInfo {
        name: "Radicle Registry Node",
        commit: "<none>",
        // commit: env!("VERGEN_SHA_SHORT"),
        // version: env!("CARGO_PKG_VERSION"),
        version: "unstable",
        executable_name: "radicle-registry",
        author: "Monadic GmbH",
        description: "Radicle Registry Node",
        support_url: "support.anonymous.an",
    };

    if let Err(e) = cli::run(::std::env::args(), cli::Exit, version) {
        eprintln!("Fatal error: {}\n\n{:?}", e, e);
        std::process::exit(1)
    }
}
