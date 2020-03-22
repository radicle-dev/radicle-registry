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
#[derive(StructOpt, Debug, Clone)]
pub enum Command {
    Register(Register),
    Unregister(Unregister),
}

#[derive(StructOpt, Debug, Clone)]
/// Register a user.
pub struct Register {
    /// Id of the user to registered. The valid charset is: 'a-z0-9-' and can't begin or end with
    /// a '-', must also not contain more than two '-' in a row.
    user_id: UserId,
}

#[async_trait::async_trait]
impl CommandT for Register {
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
pub struct Unregister {
    /// Id of the org to unregister.
    user_id: UserId,
}

#[async_trait::async_trait]
impl CommandT for Unregister {
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
