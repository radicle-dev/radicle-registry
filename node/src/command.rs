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

use sc_cli::VersionInfo;

use crate::cli;
use crate::service;

/// Parse and run command line arguments
pub fn run(version: VersionInfo) -> sc_cli::Result<()> {
    crate::logger::init();

    let args = cli::Arguments::from_args(&version);
    let mut config = sc_service::Configuration::from_version(&version);

    let chain_spec = args.chain_spec();
    let spec_factory = |_: &str| Ok(Box::new(chain_spec) as Box<_>);

    let opt_block_author = args.mine;
    let new_full_service = move |config| service::new_full(config, opt_block_author);

    match args.subcommand {
        Some(subcommand) => {
            subcommand.init(&version)?;
            subcommand.update_config(&mut config, spec_factory, &version)?;
            subcommand.run(config, service::new_for_command)
        }
        None => {
            let unsafe_rpc_external = args.unsafe_rpc_external;
            let run_cmd = args.run_cmd();
            run_cmd.init(&version)?;
            run_cmd.update_config(&mut config, spec_factory, &version)?;
            if unsafe_rpc_external {
                // Allow all hosts to connect
                config.rpc_cors = None;
            }
            run_cmd.run(config, service::new_light, new_full_service, &version)
        }
    }
}
