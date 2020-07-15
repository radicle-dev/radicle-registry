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

/// Account related commands
#[derive(StructOpt, Clone)]
pub enum Command {
    /// Show account information.
    Show(Show),
    /// Transfer funds from the author to a recipient account.
    Transfer(Transfer),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(self) -> Result<(), CommandError> {
        match self {
            Command::Show(cmd) => cmd.run().await,
            Command::Transfer(cmd) => cmd.run().await,
        }
    }
}

#[derive(StructOpt, Clone)]
pub struct Show {
    /// The account's SS58 address or the name of a local key pair.
    #[structopt(
        value_name = "address_or_name",
        parse(try_from_str = parse_account_id),
    )]
    account_id: AccountId,

    #[structopt(flatten)]
    network_options: NetworkOptions,
}

#[async_trait::async_trait]
impl CommandT for Show {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let balance = client.free_balance(&self.account_id).await?;
        println!("ss58 address: {}", self.account_id.to_ss58check());
        println!("balance: {} μRAD", balance);
        Ok(())
    }
}

#[derive(StructOpt, Clone)]
pub struct Transfer {
    // The amount to transfer.
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
                message::Transfer {
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
            "✓ Transferred {} μRAD to {} in block {}",
            self.amount, self.recipient, transfered.block,
        );
        Ok(())
    }
}
