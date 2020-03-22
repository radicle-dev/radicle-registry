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

//! Define the commands supported by the CLI related to Orgs.

use super::*;

/// Org related commands
#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    List(List),
    Show(Show),
    Transfer(Transfer),
    Register(Register),
    Unregister(Unregister),
}

#[derive(StructOpt, Debug, Clone)]
/// List all orgs in the registry
pub struct List {}

#[async_trait::async_trait]
impl CommandT for List {
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
/// Show information for a registered org.
pub struct Show {
    /// The id of the org
    org_id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for Show {
    async fn run(&self, command_context: &CommandContext) -> Result<(), CommandError> {
        let org = command_context
            .client
            .get_org(self.org_id.clone())
            .await?
            .ok_or(CommandError::OrgNotFound {
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
/// Register an org.
pub struct Register {
    /// Id of the org to register.
    org_id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for Register {
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
pub struct Unregister {
    /// Id of the org to unregister.
    org_id: OrgId,
}

#[async_trait::async_trait]
impl CommandT for Unregister {
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
/// Transfer funds from an org to a recipient.
/// The author needs to be a member of the org.
pub struct Transfer {
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
impl CommandT for Transfer {
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
