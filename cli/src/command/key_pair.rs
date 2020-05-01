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
    /// Export all or specific key-pairs from a specified file
    /// to this machine.
    Export(Export),
}

#[async_trait::async_trait]
impl CommandT for Command {
    async fn run(self) -> Result<(), CommandError> {
        match self {
            Command::Generate(cmd) => cmd.run().await,
            Command::List(cmd) => cmd.run().await,
            Command::Export(cmd) => cmd.run().await,
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

#[derive(StructOpt, Clone)]
pub struct Export {
    /// The file to import key-pairs from.
    file: std::path::PathBuf,
}

#[async_trait::async_trait]
impl CommandT for Export {

    async fn run(self) -> Result<(), CommandError> {
        // 1. List all local key pairs to help user dedice which to export
        // 2. Ask user input to select key pairs to export
        //      - :* to export all
        //      - enumerate by name, comma-separated, to export specific ones
        //          - alternatively, ask and export one at a time
        //      - :q to stop
        // 3. Add specified key-pairs to the specified file
        //      - Ask user if we should overwrite existing file if not a valid key-pairs file
        use std::io::{self, BufRead};

        List{}.run().await?;

        println!("Specify which key pairs you whish to export");
        println!("help: input '*' to import all or enumerate the specific key-pair names separated by comma");

        let mut line = String::new();
        io::stdin().lock().read_line(&mut line).unwrap();
        println!("{}", line);

        Ok(())

    }
}