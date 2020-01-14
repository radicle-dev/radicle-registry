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

use futures01::prelude::*;
use radicle_registry_client::*;
use structopt::StructOpt;

mod commands;
use commands::*;

#[derive(StructOpt, Debug, Clone)]
#[structopt(max_term_width = 80)]
struct Args {
    /// The key pair that is used to sign transaction is generated from this seed.
    #[structopt(long, default_value = "Alice", value_name = "seed")]
    author_key_seed: String,

    #[structopt(subcommand)]
    command: Command,

    /// IP address or domain name that hosts the RPC API
    #[structopt(
        long,
        default_value = "127.0.0.1",
        env = "RAD_NODE_HOST",
        parse(try_from_str = url::Host::parse),
    )]
    node_host: url::Host,
}

impl Args {
    fn command_context(&self) -> CommandContext {
        let author_key_pair =
            ed25519::Pair::from_string(format!("//{}", self.author_key_seed).as_ref(), None)
                .unwrap();
        let client = Client::create_with_executor(self.node_host.clone())
            .wait()
            .unwrap();
        CommandContext {
            author_key_pair,
            client,
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
enum Command {
    RegisterProject(RegisterProject),
    ListProjects(ListProjects),
    ShowProject(ShowProject),
    ShowGenesisHash(ShowGenesisHash),
}

fn main() {
    pretty_env_logger::init();
    let args = Args::from_args();
    let command_context = args.command_context();

    let result = match args.command {
        Command::RegisterProject(cmd) => cmd.run(&command_context),
        Command::ListProjects(cmd) => cmd.run(&command_context),
        Command::ShowProject(cmd) => cmd.run(&command_context),
        Command::ShowGenesisHash(cmd) => cmd.run(&command_context),
    };

    match result {
        Ok(_) => std::process::exit(0),
        Err(_) => std::process::exit(1),
    }
}
