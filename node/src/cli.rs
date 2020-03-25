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

//! Provides [Arguments] struct that represents the command line arguments.
use radicle_registry_runtime::AccountId;
use sc_cli::{RunCmd, Subcommand};
use structopt::{clap, StructOpt};

use crate::chain_spec::Chain;

/// Command line arguments.
///
/// Implements [StructOpt] for parsing.
#[derive(Debug, StructOpt)]
pub struct Arguments {
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,

    /// Chain to connect to.
    #[structopt(
        long,
        default_value = "dev",
        value_name = "CHAIN",
        parse(try_from_str = parse_chain),
        possible_values = &["dev", "local-devnet", "devnet"]
    )]
    pub chain: Chain,

    /// Human-readable name for this node to use for telemetry'
    #[structopt(long, value_name = "NAME")]
    name: Option<String>,

    /// Bind the RPC HTTP and WebSocket APIs to `0.0.0.0` instead of the local interface.
    #[structopt(long)]
    unsafe_rpc_external: bool,

    /// List of nodes to connect to on start'
    #[structopt(long, short, value_name = "ADDR")]
    bootnodes: Vec<String>,

    /// Where to store data
    #[structopt(long, short, value_name = "PATH")]
    data_path: Option<std::path::PathBuf>,

    /// The URL of the telemetry server to connect to.
    ///
    /// This flag can be passed multiple times as a mean to specify multiple
    /// telemetry endpoints.
    #[structopt(long, short, value_name = "URL")]
    telemetry_endpoints: Vec<String>,

    /// The secret key to use for libp2p networking provided as a hex-encoded Ed25519 32 bytes
    /// secret key.
    ///
    /// The value of this option takes precedence over `--node-key-file`.
    ///
    /// WARNING: Secrets provided as command-line arguments are easily exposed.
    /// Use of this option should be limited to development and testing. To use
    /// an externally managed secret key, use `--node-key-file` instead.
    #[structopt(long, value_name = "HEX_KEY")]
    node_key: Option<String>,

    /// The file from which to read the node's secret key to use for libp2p networking.
    ///
    /// The file must contain an unencoded 32 bytes Ed25519 secret key.
    ///
    /// If the file does not exist, it is created with a newly generated secret key.
    #[structopt(long, value_name = "FILE")]
    node_key_file: Option<std::path::PathBuf>,

    /// Account to credit block rewards and transaction fees for mined blocks.
    ///
    /// If not provided the zero account is used as the block author.
    /// Account address must be given in SS58 format.
    #[structopt(long, value_name = "SS58_ADDRESS", parse(try_from_str = parse_ss58_account_id))]
    block_author: Option<AccountId>,

    /// Bind the prometheus metrics endpoint to 0.0.0.0
    #[structopt(long)]
    prometheus_external: bool,
}

impl Arguments {
    /// Similar to [StructOpt::from_args] with additional information filled in by `version_info`.
    pub fn from_args(version_info: &sc_cli::VersionInfo) -> Self {
        let app = Arguments::clap()
            .max_term_width(80)
            .name(version_info.executable_name)
            .author(version_info.author)
            .about(version_info.description)
            // We need to manually reset the `long_about` so that `structopt` does not take the
            // code documentation of `Subcommand` for it.
            .long_about("")
            .settings(&[clap::AppSettings::UnifiedHelpMessage]);
        Arguments::from_clap(&app.get_matches())
    }

    pub fn run_cmd(self) -> RunCmd {
        // This does not panic if there are no required arguments which we statically know.
        let mut run_cmd = RunCmd::from_iter_safe(vec![] as Vec<String>).unwrap();

        let Arguments {
            bootnodes,
            data_path,
            name,
            node_key,
            node_key_file,
            telemetry_endpoints,
            unsafe_rpc_external,
            prometheus_external,
            ..
        } = self;

        run_cmd.network_config.bootnodes = bootnodes;
        run_cmd.network_config.node_key_params.node_key = node_key;
        run_cmd.network_config.node_key_params.node_key_file = node_key_file;
        run_cmd.shared_params.base_path = data_path;

        // Add verbosity 0 to all endpoints.
        let telemetry_endpoints = telemetry_endpoints
            .into_iter()
            .map(|url| (url, 0))
            .collect();

        RunCmd {
            name,
            telemetry_endpoints,
            unsafe_rpc_external,
            unsafe_ws_external: unsafe_rpc_external,
            prometheus_external,
            ..run_cmd
        }
    }

    pub fn block_author(&self) -> AccountId {
        let zero_account = AccountId::from_raw([0; 32]);
        self.block_author.unwrap_or(zero_account)
    }
}

// NOTE Update `possible_values` in the structopt attribute if something is added here.
fn parse_chain(name: &str) -> Result<Chain, String> {
    if name == "dev" {
        Ok(Chain::Dev)
    } else if name == "local-devnet" {
        Ok(Chain::DevnetLocal)
    } else if name == "devnet" {
        Ok(Chain::Devnet)
    } else {
        Err(format!("Invalid chain {}", name))
    }
}

fn parse_ss58_account_id(data: &str) -> Result<AccountId, String> {
    sp_core::crypto::Ss58Codec::from_ss58check(data).map_err(|err| format!("{:?}", err))
}
