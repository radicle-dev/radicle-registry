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

//! Define the commands supported by the CLI that
//! are not related to any specific domain.

use super::*;

/// Other commands, not related to any specific domain.
#[derive(StructOpt, Clone)]
pub enum Command {
    GenesisHash(ShowGenesisHash),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(&self) -> Result<(), CommandError> {
        match self {
            Command::GenesisHash(cmd) => cmd.run().await,
        }
    }
}

#[derive(StructOpt, Clone)]
/// Show the genesis hash the node uses
pub struct ShowGenesisHash {
    #[structopt(flatten)]
    network_options: NetworkOptions,
}

#[async_trait::async_trait]
impl CommandT for ShowGenesisHash {
    async fn run(&self) -> Result<(), CommandError> {
        let client = self.network_options.client().await?;
        let genesis_hash = client.genesis_hash();
        println!("Genesis block hash: 0x{}", hex::encode(genesis_hash));
        Ok(())
    }
}
