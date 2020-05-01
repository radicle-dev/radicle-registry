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

//! Manages key-pairs stored in the filesystem,
//! providing ways to store and retrieve them.

use directories::BaseDirs;
use sp_core::serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use thiserror::Error as ThisError;

use std::io::Error as IOError;
use std::path::PathBuf;

/// The data that is stored in the filesystem relative
/// to a key-pair. The key-pair name is used as the key
/// to this value, therefore not included here.
#[derive(Serialize, Deserialize, Clone)]
pub struct KeyPairData {
    pub seed: Seed,
}

/// The seed from which a key-pair
/// can be deterministically generated.
type Seed = [u8; 32];

#[derive(Debug, ThisError)]
pub enum Error {
    /// A key-pair with the given name already exists
    #[error("A key-pair with the given name already exists")]
    AlreadyExists(),

    /// Failed to write to the key-pairs file
    #[error("Failed to write to the key-pairs file: {0}")]
    FailedWrite(#[from] WritingError),

    /// Failed to read the key-pairs file
    #[error("Failed to read the key-pairs file: {0}")]
    FailedRead(#[from] ReadingError),

    /// Could not find a key-pair with the given name
    #[error("Could not find a key-pair with the given name")]
    NotFound(),
}

/// Possible errors when writing to the key-pairs file.
#[derive(Debug, ThisError)]
pub enum WritingError {
    #[error(transparent)]
    IO(IOError),

    #[error(transparent)]
    Serialization(serde_json::Error),
}

/// Possible errors when reading the key-pairs file.
#[derive(Debug, ThisError)]
pub enum ReadingError {
    #[error(transparent)]
    IO(IOError),

    #[error(transparent)]
    Deserialization(serde_json::Error),
}

/// Add a key-pair to the storage.
///
/// Fails if a key-pair with the given `name` already exists.
/// It can also fail from IO and Serde Json errors.
pub fn add(name: String, data: KeyPairData) -> Result<(), Error> {
    let mut key_pairs = list()?;
    if key_pairs.contains_key(&name) {
        return Err(Error::AlreadyExists());
    }

    key_pairs.insert(name, data);
    update(key_pairs)
}

/// List all the stored key-pairs.
///
/// It can fail from IO and Serde Json errors.
pub fn list() -> Result<HashMap<String, KeyPairData>, Error> {
    let path_buf = get_or_create_path()?;
    let file = File::open(path_buf.as_path()).map_err(ReadingError::IO)?;
    let key_pairs: HashMap<String, KeyPairData> =
        serde_json::from_reader(&file).map_err(ReadingError::Deserialization)?;
    Ok(key_pairs)
}

/// Get a key-pair by name.
///
/// It can fail from IO and Serde Json errors, or if no such
/// key-pair is found.
pub fn get(name: &str) -> Result<KeyPairData, Error> {
    list()?.get(name).map(Clone::clone).ok_or(Error::NotFound())
}

fn update(key_pairs: HashMap<String, KeyPairData>) -> Result<(), Error> {
    let path_buf = get_or_create_path()?;
    let new_content = serde_json::to_string(&key_pairs).map_err(WritingError::Serialization)?;
    std::fs::write(path_buf.as_path(), new_content.as_bytes()).map_err(WritingError::IO)?;
    Ok(())
}

/// The file where the user key-pairs are stored.
const FILE: &str = "key-pairs.json";

/// Get the path to the key-pairs file on disk.
///
/// If the file does not yet exist, create it and initialize
/// it with an empty object so that it can be deserialized
/// as an empty HashMap<String, KeyPairData>.
fn get_or_create_path() -> Result<PathBuf, Error> {
    let path_buf = build_path(FILE)?;

    let old_path = build_path("accounts.json")?;
    if old_path.exists() {
        println!("=> Migrating the key-pair storage to the latest version...");
        std::fs::rename(old_path, &path_buf).map_err(WritingError::IO)?;
        println!("✓ Done")
    }

    let path = path_buf.as_path();
    if !path.exists() {
        std::fs::write(path, b"{}").map_err(WritingError::IO)?;
    }

    Ok(path_buf)
}

fn build_path(filename: &str) -> Result<PathBuf, Error> {
    let dir = dir()?;
    let path = dir.join(filename);
    Ok(path)
}

fn dir() -> Result<PathBuf, Error> {
    let dir = BaseDirs::new()
        .unwrap()
        .data_dir()
        .join("radicle-registry-cli");
    std::fs::create_dir_all(&dir).map_err(ReadingError::IO)?;
    Ok(dir)
}
