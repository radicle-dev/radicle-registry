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

//! Define the command line parser and interface.

use radicle_registry_client::*;
use structopt::StructOpt;

pub mod account_storage;

mod command;
use command::{account, org, other, project, user};

/// The type that captures the command line.
#[derive(StructOpt, Clone)]
#[structopt(max_term_width = 80)]
pub struct CommandLine {
    #[structopt(subcommand)]
    pub command: Command,

    #[structopt(flatten)]
    pub options: CommandLineOptions,
}

impl CommandLine {
    pub async fn run(self) -> Result<(), CommandError> {
        let Self { command, options } = self;
        let context = CommandContext::new(options).await?;

        command.run(&context).await
    }
}

/// The accepted command-line options
#[derive(StructOpt, Clone)]
pub struct CommandLineOptions {
    /// The name of the local account to be used to sign transactions.
    #[structopt(
        long,
        env = "RAD_TX_AUTHOR",
        value_name = "account_name",
        parse(try_from_str = Self::lookup_account)
    )]
    pub tx_author: ed25519::Pair,

    /// Fee that will be charged to submit transactions.
    /// The higher the fee, the higher the priority of a transaction.
    #[structopt(long, default_value = "1", env = "RAD_TX_FEE", value_name = "fee")]
    pub tx_fee: Balance,

    /// IP address or domain name that hosts the RPC API
    #[structopt(
        long,
        default_value = "127.0.0.1",
        env = "RAD_NODE_HOST",
        parse(try_from_str = url::Host::parse),
    )]
    pub node_host: url::Host,
}

impl CommandLineOptions {
    fn lookup_account(name: &str) -> Result<ed25519::Pair, String> {
        let accounts = account_storage::list().map_err(|e| format!("{}", e))?;
        match accounts.get(&name.to_string()) {
            Some(account) => Ok(ed25519::Pair::from_seed(&account.seed)),
            None => Err(format!("Could not find local account named '{}'", name)),
        }
    }
}

/// The supported [CommandLine] commands.
/// The commands are grouped by domain.
#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    Account(account::Command),
    Org(org::Command),
    Project(project::Command),
    User(user::Command),

    #[structopt(flatten)]
    Other(other::Command),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(&self, ctx: &CommandContext) -> Result<(), CommandError> {
        match self.clone() {
            Command::Account(cmd) => cmd.run(ctx).await,
            Command::Org(cmd) => cmd.run(ctx).await,
            Command::Project(cmd) => cmd.run(ctx).await,
            Command::User(cmd) => cmd.run(ctx).await,
            Command::Other(cmd) => cmd.run(ctx).await,
        }
    }
}

/// Context for running commands.
/// Created from [CommandLineOptions].
pub struct CommandContext {
    pub tx_author: ed25519::Pair,
    pub client: Client,
    pub tx_fee: Balance,
}

impl CommandContext {
    pub async fn new(options: CommandLineOptions) -> Result<CommandContext, CommandError> {
        let client = Client::create_with_executor(options.node_host.clone()).await?;
        Ok(CommandContext {
            tx_author: options.tx_author.clone(),
            client,
            tx_fee: options.tx_fee,
        })
    }
}

/// The trait that every command must implement.
#[async_trait::async_trait]
pub trait CommandT {
    async fn run(&self, ctx: &CommandContext) -> Result<(), CommandError>;
}

/// Error returned by [CommandT::run].
///
/// Implements [From] for client errors.
#[derive(Debug, derive_more::From)]
pub enum CommandError {
    ClientError(Error),
    FailedTransaction {
        tx_hash: TxHash,
        block_hash: BlockHash,
    },
    OrgNotFound {
        org_id: OrgId,
    },
    ProjectNotFound {
        project_name: ProjectName,
        org_id: OrgId,
    },
    AccountStorageError(account_storage::Error),
}

impl core::fmt::Display for CommandError {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        match self {
            CommandError::ClientError(error) => write!(f, "Client error: {}", error),
            CommandError::FailedTransaction {
                tx_hash,
                block_hash,
            } => write!(f, "Transaction {} failed in block {}", tx_hash, block_hash),
            CommandError::OrgNotFound { org_id } => write!(f, "Cannot find org {}", org_id),
            CommandError::ProjectNotFound {
                project_name,
                org_id,
            } => write!(f, "Cannot find project {}.{}", project_name, org_id),
            CommandError::AccountStorageError(error) => {
                write!(f, "Account storage error: {}", error)
            }
        }
    }
}
