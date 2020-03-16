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

use crate::cli;
use crate::service;

/// Parse and run command line arguments
pub fn run(version: VersionInfo) -> error::Result<()> {
    crate::logger::init();

    let args = cli::Arguments::from_args(&version);
    let config = sc_service::Configuration::new(&version);

    let chain_spec = args.chain.spec();
    let spec_factory = |_: &str| Ok(Some(chain_spec));

    match args.subcommand {
        Some(subcommand) => sc_cli::run_subcommand(
            config,
            subcommand,
            spec_factory,
            |config: _| Ok(new_full_start!(config).0),
            &version,
        ),
        None => sc_cli::run(
            config,
            args.run_cmd(),
            service::new_light,
            service::new_full,
            spec_factory,
            &version,
        ),
    }
}
