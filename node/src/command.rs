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

use sc_cli::{error, VersionInfo};
use sp_consensus_aura::sr25519::AuthorityPair as AuraPair;

use crate::chain_spec;
use crate::cli::Cli;
use crate::service;

/// Parse and run command line arguments
pub fn run(version: VersionInfo) -> error::Result<()> {
    let opt = sc_cli::from_args::<Cli>(&version);

    let config = sc_service::Configuration::new(&version);

    match opt.subcommand {
        Some(subcommand) => sc_cli::run_subcommand(
            config,
            subcommand,
            chain_spec::load_spec,
            |config: _| Ok(new_full_start!(config).0),
            &version,
        ),
        None => sc_cli::run(
            config,
            opt.run,
            service::new_light,
            service::new_full,
            chain_spec::load_spec,
            &version,
        ),
    }
}
