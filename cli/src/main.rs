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

use radicle_registry_cli::*;
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
            AccountCommand::Address(cmd) => cmd.run(&command_context).await,
            AccountCommand::Balance(cmd) => cmd.run(&command_context).await,
            AccountCommand::Transfer(cmd) => cmd.run(&command_context).await,
        },
        Command::Genesis(cmd) => match cmd {
            GenesisCommand::Hash(cmd) => cmd.run(&command_context).await,
        },
        Command::Org(ocmd) => match ocmd {
            OrgCommand::Show(cmd) => cmd.run(&command_context).await,
            OrgCommand::List(cmd) => cmd.run(&command_context).await,
            OrgCommand::Register(cmd) => cmd.run(&command_context).await,
            OrgCommand::Unregister(cmd) => cmd.run(&command_context).await,
            OrgCommand::Transfer(cmd) => cmd.run(&command_context).await,
        },
        Command::Project(pcmd) => match pcmd {
            ProjectCommand::Show(cmd) => cmd.run(&command_context).await,
            ProjectCommand::List(cmd) => cmd.run(&command_context).await,
            ProjectCommand::Register(cmd) => cmd.run(&command_context).await,
        },
        Command::User(pcmd) => match pcmd {
            UserCommand::Register(cmd) => cmd.run(&command_context).await,
            UserCommand::Unregister(cmd) => cmd.run(&command_context).await,
        },
    }
}
