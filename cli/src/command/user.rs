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

//! Define the commands supported by the CLI related to Users.

use super::*;

/// User related commands
#[derive(StructOpt, Clone)]
pub enum Command {
    /// Register a user.
    Register(Register),
    /// Unregister a user.
    Unregister(Unregister),
    /// Show information for a registered user.
    Show(Show),
    /// List all users in the registry.
    List(List),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(self) -> Result<(), CommandError> {
        match self {
            user::Command::Register(cmd) => cmd.run().await,
            user::Command::Unregister(cmd) => cmd.run().await,
            user::Command::Show(cmd) => cmd.run().await,
            user::Command::List(cmd) => cmd.run().await,
        }
    }
}

#[derive(StructOpt, Clone)]
pub struct Register {
    /// Id of the user to register. The valid charset is: 'a-z0-9-' and can't begin or end with
    /// a '-', must also not contain more than two '-' in a row.
    user_id: Id,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for Register {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let register_user_fut = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::RegisterUser {
                    user_id: self.user_id.clone(),
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Registering user...");

        register_user_fut.await?.result?;
        println!("✓ User {} is now registered.", self.user_id);
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Unregister {
    /// Id of the org to unregister.
    user_id: Id,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for Unregister {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let unregister_user = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::UnregisterUser {
                    user_id: self.user_id.clone(),
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Unregistering user...");

        unregister_user.await?.result?;
        println!("✓ User {} is now unregistered.", self.user_id);
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct SetLinkUser {
    /// Id of the user.
    user_id: Id,

    /// Radicle link user revision reference
    link_user: String,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for SetLinkUser {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let link_user = Bytes128::from_vec(self.link_user.as_bytes().to_vec()).map_err(|_| {
            CommandError::InvalidLinkUser {
                link_user: self.link_user.to_owned(),
            }
        })?;
        let set_link_user = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::SetLinkUser {
                    user_id: self.user_id.clone(),
                    link_user,
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Setting link user data...");

        set_link_user.await?.result?;
        println!(
            "✓ User {} now has radicle link identity {}.",
            self.user_id, self.link_user
        );
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct ClearLinkUser {
    /// Id of the user.
    user_id: Id,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for ClearLinkUser {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let set_link_user = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::SetLinkUser {
                    user_id: self.user_id.clone(),
                    link_user: Bytes128::empty(),
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Clearing link user data...");

        set_link_user.await?.result?;
        println!("✓ User {} now has no radicle link identity.", self.user_id);
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Show {
    /// The id of the user
    user_id: Id,

    #[structopt(flatten)]
    network_options: NetworkOptions,
}

#[async_trait::async_trait]
impl CommandT for Show {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let user =
            client
                .get_user(self.user_id.clone())
                .await?
                .ok_or(CommandError::UserNotFound {
                    user_id: self.user_id.clone(),
                })?;
        let balance = client.free_balance(&user.account_id()).await?;

        println!("id: {}", self.user_id);
        println!("account id: {}", user.account_id());
        println!("balance: {} μRAD", balance);
        println!("projects: [{}]", user.projects().iter().format(", "));
        Ok(())
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
        let user_ids = client.list_users().await?;
        println!("USERS ({})", user_ids.len());
        for user_id in user_ids {
            println!("{}", user_id)
        }
        Ok(())
    }
}
