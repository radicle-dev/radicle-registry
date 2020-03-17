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
#![warn(unused_extern_crates)]

#[macro_use]
mod service_customization;
#[macro_use]
mod service;
mod chain_spec;
mod cli;
mod command;
mod logger;
mod pow;

pub use sc_cli::{error, VersionInfo};

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
        copyright_start_year: 2019,
    };

    match command::run(version) {
        Ok(_) => (),
        Err(error) => {
            log::error!("{}", error);
            std::process::exit(1);
        }
    }
}
