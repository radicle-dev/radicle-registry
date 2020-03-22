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

use radicle_registry_cli::{command::*, Args, Command, CommandError, CommandT};
use structopt::StructOpt;

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
        Command::Account(acmd) => match acmd {
            account::Command::Address(cmd) => cmd.run(&command_context).await,
            account::Command::Balance(cmd) => cmd.run(&command_context).await,
            account::Command::Transfer(cmd) => cmd.run(&command_context).await,
        },
        Command::Other(cmd) => match cmd {
            other::Command::GenesisHash(cmd) => cmd.run(&command_context).await,
        },
        Command::Org(ocmd) => match ocmd {
            org::Command::Show(cmd) => cmd.run(&command_context).await,
            org::Command::List(cmd) => cmd.run(&command_context).await,
            org::Command::Register(cmd) => cmd.run(&command_context).await,
            org::Command::Unregister(cmd) => cmd.run(&command_context).await,
            org::Command::Transfer(cmd) => cmd.run(&command_context).await,
        },
        Command::Project(pcmd) => match pcmd {
            project::Command::Show(cmd) => cmd.run(&command_context).await,
            project::Command::List(cmd) => cmd.run(&command_context).await,
            project::Command::Register(cmd) => cmd.run(&command_context).await,
        },
        Command::User(pcmd) => match pcmd {
            user::Command::Register(cmd) => cmd.run(&command_context).await,
            user::Command::Unregister(cmd) => cmd.run(&command_context).await,
        },
    }
}
