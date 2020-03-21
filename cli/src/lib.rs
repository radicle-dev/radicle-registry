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

pub mod commands;
use commands::*;

#[derive(StructOpt, Clone)]
#[structopt(max_term_width = 80)]
pub struct Args {
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
    pub author_key: ed25519::Pair,

    /// Fee that will be charged fo the transaction.
    /// The higher the fee, the higher the priority of a transaction.
    #[structopt(long, default_value = "1", env = "RAD_FEE", value_name = "fee")]
    fee: Balance,

    #[structopt(subcommand)]
    pub command: Command,

    /// IP address or domain name that hosts the RPC API
    #[structopt(
        long,
        default_value = "127.0.0.1",
        env = "RAD_NODE_HOST",
        parse(try_from_str = url::Host::parse),
    )]
    pub node_host: url::Host,
}

impl Args {
    pub async fn command_context(&self) -> Result<CommandContext, CommandError> {
        let client = Client::create_with_executor(self.node_host.clone()).await?;
        Ok(CommandContext {
            author_key_pair: self.author_key.clone(),
            client,
            fee: self.fee,
        })
    }

    fn parse_author_key(s: &str) -> Result<ed25519::Pair, String> {
        ed25519::Pair::from_string(s, None).map_err(|err| format!("{:?}", err))
    }
}

#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    Account(AccountCommand),
    Org(OrgCommand),
    Project(ProjectCommand),
    User(UserCommand),
    Genesis(GenesisCommand),
}

/// Account related commands
#[derive(StructOpt, Debug, Clone)]
pub enum AccountCommand {
    Address(ShowAddress),
    Transfer(Transfer),
    Balance(ShowBalance),
}

#[derive(StructOpt, Debug, Clone)]
/// Org related commands
pub enum OrgCommand {
    List(ListOrgs),
    Show(ShowOrg),
    Transfer(TransferOrgFunds),
    Register(RegisterOrg),
    Unregister(UnregisterOrg),
}

/// Project related commands
#[derive(StructOpt, Debug, Clone)]
pub enum ProjectCommand {
    List(ListProjects),
    Show(ShowProject),
    Register(RegisterProject),
}

/// User related commands
#[derive(StructOpt, Debug, Clone)]
pub enum UserCommand {
    Register(RegisterUser),
    Unregister(UnregisterUser),
}

/// Genesis related commands
#[derive(StructOpt, Debug, Clone)]
pub enum GenesisCommand {
    Hash(ShowGenesisHash),
}

/// Every command must implement this trait.
#[async_trait::async_trait]
pub trait CommandT {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError>;
}

/// Contextual data for running commands. Created from command line options.
pub struct CommandContext {
    pub author_key_pair: ed25519::Pair,
    pub client: Client,
    pub fee: Balance,
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
        }
    }
}
