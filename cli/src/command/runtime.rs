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

//! Define the commands supported by the CLI related to the on-chain runtime.

use super::*;

/// Project related commands
#[derive(StructOpt, Clone)]
pub enum Command {
    /// Submit a transaction to update the on-chain runtime.
    /// Requirements:
    ///   * the tx author must be the chain's sudo key
    ///   * the `spec_version` of the given wasm runtime must be greater than the chain runtime's.
    ///   * the `spec_name` must match between the wasm runtime and the chain runtime.
    Update(Update),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(self) -> Result<(), CommandError> {
        match self {
            Command::Update(cmd) => cmd.run().await,
        }
    }
}

#[derive(StructOpt, Clone)]
pub struct Update {
    /// The path to the (wasm) runtime code to submit
    path: std::path::PathBuf,

    #[structopt(flatten)]
    network_options: NetworkOptions,

    #[structopt(flatten)]
    tx_options: TxOptions,
}

#[async_trait::async_trait]
impl CommandT for Update {
    async fn run(self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let new_runtime_code =
            std::fs::read(self.path).expect("Invalid path or couldn't read the wasm file");

        let update_runtime_fut = client
            .sign_and_submit_message(
                &self.tx_options.author,
                message::UpdateRuntime {
                    code: new_runtime_code,
                },
                self.tx_options.fee,
            )
            .await?;
        announce_tx("Submitting the new on-chain runtime...");

        update_runtime_fut.await?.result?;
        println!("✓ The new on-chain runtime is now published.");
        Ok(())
    }
}
