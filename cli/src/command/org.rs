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
#[derive(StructOpt, Clone)]
pub enum Command {
    /// List all orgs in the registry
    List(List),
    /// Show information for a registered org.
    Show(Show),
    /// Transfer funds from an org to a recipient.
    /// The author needs to be a member of the org.
    Transfer(Transfer),
    /// Register an org.
    Register(Register),
    /// Unregister an org.
    Unregister(Unregister),
    /// Register a new member under an org.
    RegisterMember(RegisterMember),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(self) -> Result<(), CommandError> {
        match self {
            Command::Show(cmd) => cmd.run().await,
            Command::List(cmd) => cmd.run().await,
            Command::Register(cmd) => cmd.run().await,
            Command::Unregister(cmd) => cmd.run().await,
            Command::Transfer(cmd) => cmd.run().await,
            Command::RegisterMember(cmd) => cmd.run().await,
        }
    }
}

#[derive(StructOpt, Clone)]
pub struct List {
    #[structopt(flatten)]
    network_options: NetworkOptions,
}

#[async_trait::async_trait]
impl CommandT for List {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let org_ids = client.list_orgs().await?;
        println!("ORGS ({})", org_ids.len());
        for org_id in org_ids {
            println!("{}", org_id)
        }
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Show {
    /// The id of the org
    org_id: Id,

    #[structopt(flatten)]
    network_options: NetworkOptions,
}

#[async_trait::async_trait]
impl CommandT for Show {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let org = client
            .get_org(self.org_id.clone())
            .await?
            .ok_or(CommandError::OrgNotFound {
                org_id: self.org_id.clone(),
            })?;
        let balance = client.free_balance(&org.account_id()).await?;

        println!("id: {}", self.org_id);
        println!("account id: {}", org.account_id());
        println!("balance: {} μRAD", balance);
        println!("member ids: [{}]", org.members().iter().format(", "));
        println!("projects: [{}]", org.projects().iter().format(", "));
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Register {
    /// Id of the org to register.
    org_id: Id,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for Register {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;

        let register_org_fut = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::RegisterOrg {
                    org_id: self.org_id.clone(),
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Registering org...");

        register_org_fut.await?.result?;
        println!("✓ Org {} is now registered.", self.org_id);
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Unregister {
    /// Id of the org to unregister.
    org_id: Id,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for Unregister {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;

        let register_org_fut = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::UnregisterOrg {
                    org_id: self.org_id.clone(),
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Unregistering org...");

        register_org_fut.await?.result?;
        println!("✓ Org {} is now unregistered.", self.org_id);
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Transfer {
    /// Id of the org.
    #[structopt(value_name = "org")]
    org_id: Id,

    // The amount to transfer from the org to the recipient.
    amount: Balance,

    /// The recipient account.
    /// SS58 address or name of a local key pair.
    #[structopt(parse(try_from_str = parse_account_id))]
    recipient: AccountId,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for Transfer {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let transfer_fut = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::TransferFromOrg {
                    org_id: self.org_id.clone(),
                    recipient: self.recipient,
                    amount: self.amount,
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Transferring funds...");

        let transfered = transfer_fut.await?;
        transfered.result?;
        println!(
            "✓ Transferred {} μRAD from Org {} to Account {} in block {}",
            self.amount, self.org_id, self.recipient, transfered.block,
        );
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct RegisterMember {
    /// Id of the org to register the member under.
    org_id: Id,

    /// Id of the user to be registered as a member.
    user_id: Id,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for RegisterMember {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;

        let register_member_fut = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::RegisterMember {
                    org_id: self.org_id.clone(),
                    user_id: self.user_id.clone(),
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Registering member...");

        register_member_fut.await?.result?;
        println!(
            "✓ User {} is now a member of the Org {}.",
            self.user_id, self.org_id
        );
        Ok(())
    }
}
