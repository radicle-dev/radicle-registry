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

use radicle_registry_client::*;
use structopt::StructOpt;

mod commands;
use commands::*;

#[derive(StructOpt, Clone)]
#[structopt(max_term_width = 80)]
struct Args {
    /// Value to derive the key pair for signing transactions.
    /// See
    /// <https://substrate.dev/rustdocs/v1.0/substrate_primitives/crypto/trait.Pair.html#method.from_string>
    /// for information about the format of the string
    #[structopt(
        long,
        default_value = "//Alice",
        env = "RAD_AUTHOR_KEY",
        value_name = "key",
        parse(try_from_str = Args::parse_author_key)
    )]
    author_key: ed25519::Pair,

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
    async fn command_context(&self) -> Result<CommandContext, CommandError> {
        let client = Client::create_with_executor(self.node_host.clone()).await?;
        Ok(CommandContext {
            author_key_pair: self.author_key.clone(),
            client,
        })
    }

    fn parse_author_key(s: &str) -> Result<ed25519::Pair, String> {
        ed25519::Pair::from_string(s, None).map_err(|err| format!("{:?}", err))
    }
}

#[derive(StructOpt, Debug, Clone)]
enum Command {
    ListProjects(ListProjects),
    RegisterOrg(RegisterOrg),
    UnregisterOrg(UnregisterOrg),
    RegisterProject(RegisterProject),
    ShowBalance(ShowBalance),
    ShowGenesisHash(ShowGenesisHash),
    ShowProject(ShowProject),
    Transfer(Transfer),
    TransferOrgFunds(TransferOrgFunds),
}

#[async_std::main]
async fn main() {
    pretty_env_logger::init();
    let args = Args::from_args();
    let result = run(args).await;
    match result {
        Ok(_) => std::process::exit(0),
        Err(error) => {
            eprintln!("ERROR: {}", error);
            std::process::exit(1);
        }
    }
}

async fn run(args: Args) -> Result<(), CommandError> {
    let command_context = args.command_context().await?;

    match args.command {
        Command::ListProjects(cmd) => cmd.run(&command_context).await,
        Command::RegisterOrg(cmd) => cmd.run(&command_context).await,
        Command::UnregisterOrg(cmd) => cmd.run(&command_context).await,
        Command::RegisterProject(cmd) => cmd.run(&command_context).await,
        Command::ShowBalance(cmd) => cmd.run(&command_context).await,
        Command::ShowGenesisHash(cmd) => cmd.run(&command_context).await,
        Command::ShowProject(cmd) => cmd.run(&command_context).await,
        Command::Transfer(cmd) => cmd.run(&command_context).await,
        Command::TransferOrgFunds(cmd) => cmd.run(&command_context).await,
    }
}
