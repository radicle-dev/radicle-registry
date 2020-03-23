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
#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    Address(ShowAddress),
    Balance(ShowBalance),
    Transfer(Transfer),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(&self, ctx: &CommandContext) -> Result<(), CommandError> {
        match self {
            Command::Address(cmd) => cmd.run(ctx).await,
            Command::Balance(cmd) => cmd.run(ctx).await,
            Command::Transfer(cmd) => cmd.run(ctx).await,
        }
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
    async fn run(&self, _ctx: &CommandContext) -> Result<(), CommandError> {
        let key_pair =
            ed25519::Pair::from_string(format!("//{}", self.seed).as_str(), None).unwrap();
        println!("SS58 address: {}", key_pair.public().to_ss58check());
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
    async fn run(&self, ctx: &CommandContext) -> Result<(), CommandError> {
        let balance = ctx.client.free_balance(&self.account_id).await?;
        println!("{} RAD", balance);
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

#[async_trait::async_trait]
impl CommandT for Transfer {
    async fn run(&self, ctx: &CommandContext) -> Result<(), CommandError> {
        let client = &ctx.client;

        let transfer_fut = client
            .sign_and_submit_message(
                &ctx.tx_author,
                message::Transfer {
                    recipient: self.recipient,
                    balance: self.funds,
                },
                ctx.tx_fee,
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
