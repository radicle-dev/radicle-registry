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

//! Define the commands supported by the CLI related to Accounts.

use super::*;
use crate::account_storage;

/// Account related commands
#[derive(StructOpt, Clone)]
pub enum Command {
    /// Show the balance of an account.
    Balance(ShowBalance),
    /// Generate a local account and store it on disk.
    /// Fail if there is already an account with the given `name`
    Generate(Generate),
    /// List all the local accounts.
    List(List),
    /// Transfer funds from the author to a recipient account.
    Transfer(Transfer),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(self) -> Result<(), CommandError> {
        match self {
            Command::Balance(cmd) => cmd.run().await,
            Command::Generate(cmd) => cmd.run().await,
            Command::List(cmd) => cmd.run().await,
            Command::Transfer(cmd) => cmd.run().await,
        }
    }
}

/// Show the balance of an account
#[derive(StructOpt, Clone)]
pub struct ShowBalance {
    /// SS58 address or name of a local account.
    #[structopt(
        value_name = "account",
        parse(try_from_str = parse_account_id),
    )]
    account_id: AccountId,

    #[structopt(flatten)]
    network_options: NetworkOptions,
}

#[async_trait::async_trait]
impl CommandT for ShowBalance {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let balance = client.free_balance(&self.account_id).await?;
        println!("{} μRAD", balance);
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Generate {
    /// The name that uniquely identifies the account locally.
    name: String,
}

#[async_trait::async_trait]
impl CommandT for Generate {
    async fn run(self) -> Result<(), CommandError> {
        let (key_pair, seed) = ed25519::Pair::generate();
        account_storage::add(self.name, account_storage::AccountData { seed })?;
        println!("✓ Account generated successfully");
        println!("ℹ SS58 address: {}", key_pair.public().to_ss58check());
        Ok(())
    }
}
#[derive(StructOpt, Clone)]
pub struct List {}

#[async_trait::async_trait]
impl CommandT for List {
    async fn run(self) -> Result<(), CommandError> {
        let accounts = account_storage::list()?;

        println!("Accounts ({})", accounts.len());
        for (name, data) in accounts {
            println!("Account '{}'", name);
            println!(
                "\tSS58 address: {}",
                ed25519::Pair::from_seed(&data.seed).public().to_ss58check()
            );
        }
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Transfer {
    // The amount to transfer.
    amount: Balance,

    /// The recipient account.
    /// SS58 address or name of a local account.
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
                message::Transfer {
                    recipient: self.recipient,
                    balance: self.amount,
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Transferring funds...");

        let transfered = transfer_fut.await?;
        transfered.result?;
        println!(
            "✓ Transferred {} μRAD to {} in block {}",
            self.amount, self.recipient, transfered.block,
        );
        Ok(())
    }
}
