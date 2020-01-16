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

//! Defines [CommandT] trait, structs for all commands and their [CommandT] implementations.
use radicle_registry_client::*;
use std::str::FromStr;
use structopt::StructOpt;

lazy_static::lazy_static! {
    static ref RAD_DOMAIN: String32 =
        String32::from_str("rad").expect("statically valid");
}

/// Contextual data for running commands. Created from command line options.
pub struct CommandContext {
    pub author_key_pair: ed25519::Pair,
    pub client: Client,
}

/// Every CLI command must implement this trait.
#[async_trait::async_trait]
pub trait CommandT {
    async fn run(&self, command_context: &CommandContext) -> Result<(), ()>;
}

#[derive(StructOpt, Debug, Clone)]
/// Show information for a registered project in the .rad domain.
pub struct ShowProject {
    project_name: String,
}

#[async_trait::async_trait]
impl CommandT for ShowProject {
    async fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let project_name: String32 = match self.project_name.parse() {
            Ok(project_name) => project_name,
            Err(error) => {
                println!("Invalid project name: {}", error);
                return Err(());
            }
        };
        let opt_project = command_context
            .client
            .get_project((project_name.clone(), RAD_DOMAIN.clone()))
            .await
            .unwrap();

        let project = match opt_project {
            None => {
                println!("Project {}.{} not found", project_name, RAD_DOMAIN.clone());
                return Err(());
            }
            Some(project) => project,
        };

        let balance = command_context
            .client
            .free_balance(&project.account_id)
            .await
            .unwrap();

        println!("project: {}.{}", project.id.0, project.id.1);
        println!("account id: {}", project.account_id);
        println!("balance: {}", balance);
        println!("checkpoint: {}", project.current_cp);
        println!("members: {:?}", project.members);
        Ok(())
    }
}
#[derive(StructOpt, Debug, Clone)]
/// List all projects in the registry
pub struct ListProjects {}

#[async_trait::async_trait]
impl CommandT for ListProjects {
    async fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let project_ids = command_context.client.list_projects().await.unwrap();
        println!("PROJECTS");
        for (name, domain) in project_ids {
            println!("{}.{}", name, domain)
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Register a project under the default "rad" domain.
pub struct RegisterProject {
    /// Name of the project to register.
    name: String32,
    /// Project state hash. A hex-encoded 32 byte string. Defaults to all zeros.
    project_hash: Option<H256>,
}

#[async_trait::async_trait]
impl CommandT for RegisterProject {
    async fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let client = &command_context.client;

        let create_checkpoint_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::CreateCheckpoint {
                    project_hash: self.project_hash.unwrap_or_default(),
                    previous_checkpoint_id: None,
                },
            )
            .await
            .unwrap();
        println!("creating checkpoint...");

        let checkpoint_created = create_checkpoint_fut.await.unwrap();
        let checkpoint_id = checkpoint_created.result.unwrap();
        println!("checkpoint created in block {}", checkpoint_created.block,);

        let domain = String32::from_str("rad").expect("statically valid");
        let project_id = (self.name.clone(), domain.clone());
        let register_project_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::RegisterProject {
                    id: project_id,
                    checkpoint_id,
                    metadata: Vec::new(),
                },
            )
            .await
            .unwrap();
        println!("registering project...");
        let project_registered = register_project_fut.await.unwrap();
        match project_registered.result {
            Ok(()) => {
                println!(
                    "project {}.{} registered in block {}",
                    self.name, domain, project_registered.block,
                );
                Ok(())
            }
            Err(_) => {
                println!("transaction failed in block {}", project_registered.block,);
                Err(())
            }
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Show the genesis hash the node uses
pub struct ShowGenesisHash {}

#[async_trait::async_trait]
impl CommandT for ShowGenesisHash {
    async fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let genesis_hash = command_context.client.genesis_hash();
        println!("Gensis block hash: 0x{}", hex::encode(genesis_hash));
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Transfer funds to recipient
pub struct Transfer {
    #[structopt(parse(try_from_str = parse_account_id))]
    /// Recipient Account in SS58 address format
    recipient: AccountId,
    funds: Balance,
}

fn parse_account_id(data: &str) -> Result<AccountId, String> {
    sp_core::crypto::Ss58Codec::from_ss58check(data).map_err(|err| format!("{:?}", err))
}

#[async_trait::async_trait]
impl CommandT for Transfer {
    async fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let client = &command_context.client;

        let transfer_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::Transfer {
                    recipient: self.recipient,
                    balance: self.funds,
                },
            )
            .await
            .unwrap();
        println!("transferring funds...");
        let project_registered = transfer_fut.await.unwrap();
        match project_registered.result {
            Ok(()) => {
                println!(
                    "transferred {} RAD to {} in block {}",
                    self.funds, self.recipient, project_registered.block,
                );
                Ok(())
            }
            Err(_) => {
                println!("transaction failed in block {}", project_registered.block,);
                Err(())
            }
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Transfer funds from a project to a recipient. The author needs to be the owner of the project
pub struct TransferProjectFunds {
    /// Name of the project in the .rad domain
    #[structopt(value_name = "project")]
    project_name: ProjectName,

    /// Recipient Account in SS58 address format
    #[structopt(parse(try_from_str = parse_account_id))]
    recipient: AccountId,
    funds: Balance,
}

#[async_trait::async_trait]
impl CommandT for TransferProjectFunds {
    async fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let client = &command_context.client;
        let transfer_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::TransferFromProject {
                    project: (self.project_name.clone(), RAD_DOMAIN.clone()),
                    recipient: self.recipient,
                    value: self.funds,
                },
            )
            .await
            .unwrap();
        println!("transferring funds...");
        let project_registered = transfer_fut.await.unwrap();
        match project_registered.result {
            Ok(()) => {
                println!(
                    "transferred {} RAD from {}.{} to {} in block {}",
                    self.funds,
                    self.project_name,
                    RAD_DOMAIN.clone(),
                    self.recipient,
                    project_registered.block,
                );
                Ok(())
            }
            Err(_) => {
                println!("transaction failed in block {}", project_registered.block);
                Err(())
            }
        }
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Show the balance of an account
pub struct ShowBalance {
    #[structopt(
        value_name = "account",
        parse(try_from_str = parse_account_id),
    )]
    /// SS58 address
    account_id: AccountId,
}

#[async_trait::async_trait]
impl CommandT for ShowBalance {
    async fn run(&self, command_context: &CommandContext) -> Result<(), ()> {
        let balance = command_context
            .client
            .free_balance(&self.account_id)
            .await
            .unwrap();
        println!("{} RAD", balance);
        Ok(())
    }
}
