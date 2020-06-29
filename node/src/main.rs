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

//! Substrate Node Template CLI library.

#![warn(missing_docs)]

mod blockchain;
mod chain_spec;
mod cli;
mod logger;
mod metrics;
mod pow;
mod service;

use crate::cli::Cli;
use sc_cli::SubstrateCli;

fn main() {
    match Cli::from_args().run() {
        Ok(_) => (),
        Err(error) => {
            log::error!("{}", error);
            std::process::exit(1);
        }
    }
}
