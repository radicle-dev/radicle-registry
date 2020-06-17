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

#![allow(clippy::large_enum_variant)]

use lazy_static::lazy_static;
use radicle_registry_client::*;
use structopt::StructOpt;
use thiserror::Error as ThisError;

pub mod key_pair_storage;

mod command;
use command::{account, key_pair, org, other, project, runtime, user};

/// The type that captures the command line.
#[derive(StructOpt, Clone)]
#[structopt(
    max_term_width = 80,
    version = env!("VERGEN_SEMVER"),
)]
pub struct CommandLine {
    #[structopt(subcommand)]
    pub command: Command,
}

impl CommandLine {
    pub async fn run(self) -> Result<(), CommandError> {
        self.command.run().await
    }
}

/// Network-related command-line options
#[derive(StructOpt, Clone, Debug)]
pub struct NetworkOptions {
    /// IP address or domain name that hosts the RPC API
    #[structopt(
        long,
        default_value = "rpc.ff.radicle.network",
        env = "RAD_NODE_HOST",
        parse(try_from_str = Self::parse_node_host),
    )]
    pub node_host: url::Host,
}

impl NetworkOptions {
    fn parse_node_host(value: &str) -> Result<url::Host, url::ParseError> {
        let node_host = match value {
            // "localhost" gets translated to its ipv6 version in some
            // systems, which causes the client-node rpc connection to
            // fail as ipv6 is not yet supported.
            "localhost" => "127.0.0.1",
            x => x,
        };
        url::Host::parse(node_host)
    }

    pub async fn client(&self) -> Result<Client, Error> {
        Client::create_with_executor(self.node_host.clone()).await
    }
}

/// Transaction-related command-line options
#[derive(StructOpt, Clone)]
pub struct TxOptions {
    /// The name of the local key-pair to be used to sign transactions.
    #[structopt(
        long,
        env = "RAD_AUTHOR",
        value_name = "key_pair_name",
        parse(try_from_str = lookup_key_pair)
    )]
    pub author: ed25519::Pair,

    /// Fee that will be charged to submit transactions.
    /// The higher the fee, the higher the priority of a transaction.
    #[structopt(long, default_value = &FEE_DEFAULT, env = "RAD_FEE", value_name = "fee")]
    pub fee: Balance,
}

lazy_static! {
    static ref FEE_DEFAULT: String = MINIMUM_FEE.to_string();
}

fn lookup_key_pair(name: &str) -> Result<ed25519::Pair, String> {
    key_pair_storage::get(name)
        .map(|data| ed25519::Pair::from_seed(&data.seed))
        .map_err(|e| format!("{}", e))
}

/// The supported [CommandLine] commands.
/// The commands are grouped by domain.
#[derive(StructOpt, Clone)]
pub enum Command {
    Account(account::Command),
    KeyPair(key_pair::Command),
    Org(org::Command),
    Project(project::Command),
    Runtime(runtime::Command),
    User(user::Command),

    #[structopt(flatten)]
    Other(other::Command),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(self) -> Result<(), CommandError> {
        match self.clone() {
            Command::Account(cmd) => cmd.run().await,
            Command::KeyPair(cmd) => cmd.run().await,
            Command::Org(cmd) => cmd.run().await,
            Command::Project(cmd) => cmd.run().await,
            Command::User(cmd) => cmd.run().await,
            Command::Runtime(cmd) => cmd.run().await,
            Command::Other(cmd) => cmd.run().await,
        }
    }
}

/// The trait that every command must implement.
#[async_trait::async_trait]
pub trait CommandT {
    async fn run(self) -> Result<(), CommandError>;
}

/// Error returned by [CommandT::run].
///
/// Implements [From] for client errors and [key_pair_storage] errors.
#[derive(Debug, ThisError)]
pub enum CommandError {
    #[error("client error")]
    ClientError(#[from] Error),

    #[error(transparent)]
    FailedTransaction(#[from] TransactionError),

    #[error("cannot find org {org_id}")]
    OrgNotFound { org_id: Id },

    #[error("cannot find user {user_id}")]
    UserNotFound { user_id: Id },

    #[error("cannot find project {project_name}.{project_registrant:?}")]
    ProjectNotFound {
        project_name: ProjectName,
        project_registrant: ProjectRegistrant,
    },

    #[error(transparent)]
    KeyPairStorageError(#[from] key_pair_storage::Error),
}
