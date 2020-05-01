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

//! Define the commands supported by the CLI related to key-pairs.

use super::*;
use crate::key_pair_storage;

/// Key-pair related commands
#[derive(StructOpt, Clone)]
pub enum Command {
    /// Generate a random key-pair identified by `name` and
    /// store it on disk. Fail if there is already a key-pair
    /// with the given `name`.
    Generate(Generate),
    /// List all the local key pairs.
    List(List),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(self) -> Result<(), CommandError> {
        match self {
            Command::Generate(cmd) => cmd.run().await,
            Command::List(cmd) => cmd.run().await,
        }
    }
}

#[derive(StructOpt, Clone)]
pub struct Generate {
    /// The name that uniquely identifies the key-pair locally.
    name: String,
}

#[async_trait::async_trait]
impl CommandT for Generate {
    async fn run(self) -> Result<(), CommandError> {
        let (key_pair, seed) = ed25519::Pair::generate();
        key_pair_storage::add(self.name, key_pair_storage::KeyPairData { seed })?;
        println!("✓ Key-pair generated successfully");
        println!("ⓘ SS58 address: {}", key_pair.public().to_ss58check());
        Ok(())
    }
}
#[derive(StructOpt, Clone)]
pub struct List {}

#[async_trait::async_trait]
impl CommandT for List {
    async fn run(self) -> Result<(), CommandError> {
        let key_pairs = key_pair_storage::list()?;
        println!("Key-pairs ({})\n", key_pairs.len());
        for (name, data) in key_pairs {
            println!("  '{}'", name);
            println!(
                "  ss58 address: {}\n",
                ed25519::Pair::from_seed(&data.seed).public().to_ss58check()
            );
        }
        Ok(())
    }
}
