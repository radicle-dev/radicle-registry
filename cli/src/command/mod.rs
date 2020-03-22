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

//! Define the commands supported by the CLI.

use crate::{CommandContext, CommandError, CommandT};
use radicle_registry_client::*;

use sp_core::crypto::Ss58Codec;
use structopt::StructOpt;

pub mod account;
pub mod org;
pub mod other;
pub mod project;
pub mod user;

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

fn parse_account_id(data: &str) -> Result<AccountId, String> {
    Ss58Codec::from_ss58check(data).map_err(|err| format!("{:?}", err))
}
