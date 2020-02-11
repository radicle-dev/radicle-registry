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
use structopt::StructOpt;

/// Contextual data for running commands. Created from command line options.
pub struct CommandContext {
    pub author_key_pair: ed25519::Pair,
    pub client: Client,
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
    ProjectNotFound {
        name: ProjectName,
        domain: ProjectDomain,
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
            CommandError::ProjectNotFound { name, domain } => {
                write!(f, "Cannot find project {}.{}", name, domain)
            }
        }
    }
}

/// Check that a transaction has been applied succesfully.
///
/// If the transaction failed, that is if `tx_applied.result` is `Err`, then we return a
/// [CommandError]. Otherwise we return the `Ok` value of the transaction result.
fn transaction_applied_ok<Message_, T, E>(
    tx_applied: &TransactionApplied<Message_>,
) -> Result<T, CommandError>
where
    Message_: Message<Result = Result<T, E>>,
    T: Copy + Send + 'static,
    E: Send + 'static,
{
    match tx_applied.result {
        Ok(value) => Ok(value),
        Err(_) => Err(CommandError::FailedTransaction {
            tx_hash: tx_applied.tx_hash,
            block_hash: tx_applied.block,
        }),
    }
}

/// Every CLI command must implement this trait.
#[async_trait::async_trait]
pub trait CommandT {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError>;
}

#[derive(StructOpt, Debug, Clone)]
/// Show information for a registered project in the .rad domain.
pub struct ShowProject {
    project_name: String32,
}

#[async_trait::async_trait]
impl CommandT for ShowProject {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let project_domain = ProjectDomain::rad_domain();
        let opt_project = command_context
            .client
            .get_project((self.project_name.clone(), project_domain.clone()))
            .await?;

        let project = match opt_project {
            None => {
                return Err(CommandError::ProjectNotFound {
                    name: self.project_name.clone(),
                    domain: project_domain,
                });
            }
            Some(project) => project,
        };

        let balance = command_context
            .client
            .free_balance(&project.account_id)
            .await?;

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
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let project_ids = command_context.client.list_projects().await?;
        println!("PROJECTS");
        for (name, domain) in project_ids {
            println!("{}.{}", name, domain)
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Register an org.
pub struct RegisterOrg {
    /// Id of the org to register.
    id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for RegisterOrg {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;

        let register_org_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::RegisterOrg {
                    id: self.id.clone(),
                },
            )
            .await?;
        println!("Registering org...");

        let org_registered = register_org_fut.await?;
        transaction_applied_ok(&org_registered)?;
        println!("Org {} is now registered.", self.id);
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Unregister an org.
pub struct UnregisterOrg {
    /// Id of the org to unregister.
    id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for UnregisterOrg {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;

        let register_org_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::UnregisterOrg {
                    id: self.id.clone(),
                },
            )
            .await?;
        println!("Unregistering org...");

        let org_unregistered = register_org_fut.await?;
        transaction_applied_ok(&org_unregistered)?;
        println!("Org {} is now unregistered.", self.id);
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
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;

        let create_checkpoint_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::CreateCheckpoint {
                    project_hash: self.project_hash.unwrap_or_default(),
                    previous_checkpoint_id: None,
                },
            )
            .await?;
        println!("creating checkpoint...");

        let checkpoint_created = create_checkpoint_fut.await?;
        let checkpoint_id = transaction_applied_ok(&checkpoint_created)?;
        println!("checkpoint created in block {}", checkpoint_created.block);

        let project_id: ProjectId = (self.name.clone(), ProjectDomain::rad_domain());
        let register_project_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::RegisterProject {
                    id: project_id,
                    checkpoint_id,
                    metadata: Bytes128::random(),
                },
            )
            .await?;
        println!("registering project...");
        let project_registered = register_project_fut.await?;
        transaction_applied_ok(&project_registered)?;
        println!(
            "project {}.{} registered in block {}",
            self.name,
            ProjectDomain::rad_domain(),
            project_registered.block,
        );
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Show the genesis hash the node uses
pub struct ShowGenesisHash {}

#[async_trait::async_trait]
impl CommandT for ShowGenesisHash {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
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
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;

        let transfer_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::Transfer {
                    recipient: self.recipient,
                    balance: self.funds,
                },
            )
            .await?;
        println!("transferring funds...");
        let transfered = transfer_fut.await?;
        transaction_applied_ok(&transfered)?;
        println!(
            "transferred {} RAD to {} in block {}",
            self.funds, self.recipient, transfered.block,
        );
        Ok(())
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
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;
        let transfer_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::TransferFromProject {
                    project: (self.project_name.clone(), ProjectDomain::rad_domain()),
                    recipient: self.recipient,
                    value: self.funds,
                },
            )
            .await?;
        println!("transferring funds...");
        let transfered = transfer_fut.await?;
        transaction_applied_ok(&transfered)?;
        println!(
            "transferred {} RAD from {}.{} to {} in block {}",
            self.funds,
            self.project_name,
            ProjectDomain::rad_domain(),
            self.recipient,
            transfered.block,
        );
        Ok(())
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
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let balance = command_context
            .client
            .free_balance(&self.account_id)
            .await?;
        println!("{} RAD", balance);
        Ok(())
    }
}
