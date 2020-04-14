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

//! The executable entry point for the Radicle Registry CLI.

use radicle_registry_cli::CommandLine;
use std::error::Error;
use structopt::StructOpt;

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    let cmd_line = CommandLine::from_args();
    let result = cmd_line.run().await;

    match result {
        Ok(_) => std::process::exit(0),
        Err(error) => {
            print_error(&error);
            std::process::exit(1);
        }
    }
}

fn print_error(mut error: &dyn Error) {
    eprintln!("Error: {}", error);
    while let Some(source) = error.source() {
        error = source;
        eprintln!("  Caused by: {}", error);
    }
}
