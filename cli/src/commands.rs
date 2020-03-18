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

use sp_core::crypto::Ss58Codec;

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
/// Show information for a registered project.
pub struct ShowProject {
    /// The name of the project
    project_name: ProjectName,
    /// The org in which the project is registered.
    org_id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for ShowProject {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let opt_project = command_context
            .client
            .get_project(self.project_name.clone(), self.org_id.clone())
            .await?;

        let project = match opt_project {
            None => {
                return Err(CommandError::ProjectNotFound {
                    project_name: self.project_name.clone(),
                    org_id: self.org_id.clone(),
                });
            }
            Some(project) => project,
        };

        println!("project: {}.{}", project.name, project.org_id);
        println!("checkpoint: {}", project.current_cp);
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Show information for a registered org.
pub struct ShowOrg {
    /// The id of the org
    org_id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for ShowOrg {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let opt_org = command_context.client.get_org(self.org_id.clone()).await?;

        let org = opt_org.ok_or(CommandError::OrgNotFound {
            org_id: self.org_id.clone(),
        })?;

        println!("id: {}", org.id.clone());
        println!("account_id: {}", org.account_id.clone());
        println!("members: {:?}", org.members.clone());
        println!("projects: {:?}", org.projects);
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// List all orgs in the registry
pub struct ListOrgs {}

#[async_trait::async_trait]
impl CommandT for ListOrgs {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let org_ids = command_context.client.list_orgs().await?;
        println!("ORGS ({})", org_ids.len());
        for org_id in org_ids {
            println!("{}", org_id)
        }
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
        println!("PROJECTS ({})", project_ids.len());
        for (name, org) in project_ids {
            println!("{}.{}", name, org)
        }
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Register an org.
pub struct RegisterOrg {
    /// Id of the org to register.
    org_id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for RegisterOrg {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;

        let register_org_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::RegisterOrg {
                    org_id: self.org_id.clone(),
                },
                command_context.fee,
            )
            .await?;
        println!("Registering org...");

        let org_registered = register_org_fut.await?;
        transaction_applied_ok(&org_registered)?;
        println!("Org {} is now registered.", self.org_id);
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Unregister an org.
pub struct UnregisterOrg {
    /// Id of the org to unregister.
    org_id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for UnregisterOrg {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;

        let register_org_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::UnregisterOrg {
                    org_id: self.org_id.clone(),
                },
                command_context.fee,
            )
            .await?;
        println!("Unregistering org...");

        let org_unregistered = register_org_fut.await?;
        transaction_applied_ok(&org_unregistered)?;
        println!("Org {} is now unregistered.", self.org_id);
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Register a project with the given name under the given org.
pub struct RegisterProject {
    /// Name of the project to register.
    project_name: ProjectName,

    /// Org under which to register the project.
    org_id: OrgId,

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
                command_context.fee,
            )
            .await?;
        println!("creating checkpoint...");

        let checkpoint_created = create_checkpoint_fut.await?;
        let checkpoint_id = transaction_applied_ok(&checkpoint_created)?;
        println!("checkpoint created in block {}", checkpoint_created.block);

        let register_project_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::RegisterProject {
                    project_name: self.project_name.clone(),
                    org_id: self.org_id.clone(),
                    checkpoint_id,
                    metadata: Bytes128::random(),
                },
                command_context.fee,
            )
            .await?;
        println!("registering project...");
        let project_registered = register_project_fut.await?;
        transaction_applied_ok(&project_registered)?;
        println!(
            "project {}.{} registered in block {}",
            self.project_name, self.org_id, project_registered.block,
        );
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Register a user.
pub struct RegisterUser {
    /// Id of the user to registered. The valid charset is: 'a-z0-9-' and can't begin or end with
    /// a '-', must also not contain more than two '-' in a row.
    user_id: UserId,
}

#[async_trait::async_trait]
impl CommandT for RegisterUser {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;

        let register_user_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::RegisterUser {
                    user_id: self.user_id.clone(),
                },
                command_context.fee,
            )
            .await?;
        println!("Registering user...");

        let user_registered = register_user_fut.await?;
        transaction_applied_ok(&user_registered)?;
        println!("User {} is now registered.", self.user_id);
        Ok(())
    }
}

#[derive(StructOpt, Debug, Clone)]
/// Unregister a user.
pub struct UnregisterUser {
    /// Id of the org to unregister.
    user_id: UserId,
}

#[async_trait::async_trait]
impl CommandT for UnregisterUser {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;

        let unregister_user = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::UnregisterUser {
                    user_id: self.user_id.clone(),
                },
                command_context.fee,
            )
            .await?;
        println!("Unregistering user...");

        let user_unregistered = unregister_user.await?;
        transaction_applied_ok(&user_unregistered)?;
        println!("User {} is now unregistered.", self.user_id);
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
    /// Recipient Account in SS58 address format.
    recipient: AccountId,
    // The amount to transfer.
    funds: Balance,
}

fn parse_account_id(data: &str) -> Result<AccountId, String> {
    Ss58Codec::from_ss58check(data).map_err(|err| format!("{:?}", err))
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
                command_context.fee,
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
/// Transfer funds from an org to a recipient.
/// The author needs to be a member of the org.
pub struct TransferOrgFunds {
    /// Id of the org.
    #[structopt(value_name = "org")]
    org_id: OrgId,

    /// Recipient Account in SS58 address format
    #[structopt(parse(try_from_str = parse_account_id))]
    recipient: AccountId,

    // The balance to transfer from the org to the recipient.
    funds: Balance,
}

#[async_trait::async_trait]
impl CommandT for TransferOrgFunds {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let client = &command_context.client;
        let transfer_fut = client
            .sign_and_submit_message(
                &command_context.author_key_pair,
                message::TransferFromOrg {
                    org_id: self.org_id.clone(),
                    recipient: self.recipient,
                    value: self.funds,
                },
                command_context.fee,
            )
            .await?;
        println!("transferring funds...");
        let transfered = transfer_fut.await?;
        transaction_applied_ok(&transfered)?;
        println!(
            "transferred {} RAD from Org {} to Account {} in block {}",
            self.funds, self.org_id, self.recipient, transfered.block,
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

#[derive(StructOpt, Debug, Clone)]
/// Show the SS58 address for the key pair derived from `seed`.
///
/// For more information on how the seed string is interpreted see
/// <https://substrate.dev/rustdocs/v1.0/substrate_primitives/crypto/trait.Pair.html#method.from_string>.
pub struct ShowAddress {
    seed: String,
}

#[async_trait::async_trait]
impl CommandT for ShowAddress {
    async fn run(&self, _command_context: &CommandContext) -> Result<(), CommandError> {
        let key_pair =
            ed25519::Pair::from_string(format!("//{}", self.seed).as_str(), None).unwrap();
        println!("SS58 address: {}", key_pair.public().to_ss58check());
        Ok(())
    }
}
