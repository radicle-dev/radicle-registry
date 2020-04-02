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

//! Manages accounts stored in the filesystem,
//! providing ways to store and retrieve them.

use directories::BaseDirs;
use sp_core::serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use thiserror::Error as ThisError;

use std::io::Error as IOError;
use std::path::PathBuf;

/// The data that is stored in the filesystem relative
/// to an account. The account name is used as the key
/// to this value, therefore not included here.
#[derive(Serialize, Deserialize)]
pub struct AccountData {
    pub seed: Seed,
}

/// The seed from which an account's key pair
/// can be determiniscally generated.
type Seed = [u8; 32];

#[derive(Debug, ThisError)]
pub enum Error {
    /// An account with the given name already exists
    #[error("An account with the given name already exists")]
    AlreadyExists(),

    /// Failed to write to the accounts file
    #[error("Failed to write to the accounts file: {0}")]
    FailedWrite(#[from] WritingError),

    /// Failed to read the accounts file
    #[error("Failed to read the accounts file: {0}")]
    FailedRead(#[from] ReadingError),
}

/// Possible errors when writing to the accounts file.
#[derive(Debug, ThisError)]
pub enum WritingError {
    #[error(transparent)]
    IO(IOError),

    #[error(transparent)]
    Serialization(serde_json::Error),
}

/// Possible errors when reading the accounts file.
#[derive(Debug, ThisError)]
pub enum ReadingError {
    #[error(transparent)]
    IO(IOError),

    #[error(transparent)]
    Deserialization(serde_json::Error),
}

/// Add an account to the storage.
///
/// Fails if an account with the given `name` already exists.
/// It can also fail from IO and Serde Json errors.
pub fn add(name: String, data: AccountData) -> Result<(), Error> {
    let mut accounts = list()?;
    if accounts.contains_key(&name) {
        return Err(Error::AlreadyExists());
    }

    accounts.insert(name, data);
    update(accounts)
}

/// List all the stored accounts.
///
/// It can fail from IO and Serde Json errors.
pub fn list() -> Result<HashMap<String, AccountData>, Error> {
    let path_buf = get_or_create_path()?;
    let file = File::open(path_buf.as_path()).map_err(ReadingError::IO)?;
    let accounts: HashMap<String, AccountData> =
        serde_json::from_reader(&file).map_err(ReadingError::Deserialization)?;
    Ok(accounts)
}

fn update(accounts: HashMap<String, AccountData>) -> Result<(), Error> {
    let path_buf = get_or_create_path()?;
    let new_content = serde_json::to_string(&accounts).map_err(WritingError::Serialization)?;
    std::fs::write(path_buf.as_path(), new_content.as_bytes()).map_err(WritingError::IO)?;
    Ok(())
}

const FILE: &str = "accounts.json";

// Get the path to the accounts file on disk.
//
// If the file does not yet exist, create it and initialize
// it with an empty object so that it can be deserialized
// as an empty HashMap<String, AccountData>.
fn get_or_create_path() -> Result<PathBuf, Error> {
    let dir = dir()?;
    let path_buf = dir.join(FILE);
    let path = path_buf.as_path();

    if !path.exists() {
        std::fs::write(path, b"{}").map_err(WritingError::IO)?;
    }

    Ok(path_buf)
}

fn dir() -> Result<PathBuf, Error> {
    let dir = BaseDirs::new()
        .unwrap()
        .data_dir()
        .join("radicle-registry-cli");
    std::fs::create_dir_all(&dir).map_err(ReadingError::IO)?;
    Ok(dir)
}
