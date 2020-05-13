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

//! Manages key pairs stored in the filesystem,
//! providing ways to store and retrieve them.

use directories::BaseDirs;
use sp_core::serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use thiserror::Error as ThisError;

use lazy_static::lazy_static;
use std::io::Error as IOError;
use std::path::{Path, PathBuf};

lazy_static! {
    /// The file where the key pairs are stored.
    static ref FILE: PathBuf = build_path("key-pairs.json");
}

/// The possible file variants to be handled when deserializing FILE.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
enum KeyStorageFile {
    /// The genesis, unversioned file variant.
    Unversioned(HashMap<String, KeyPairData>),

    /// A versioned file variant, to which we have moved to
    /// in order to leverage backwards-compatibility.
    Versioned(VersionedFile),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "version")]
enum VersionedFile {
    #[serde(rename = "1")]
    V1 {
        key_pairs: HashMap<String, KeyPairData>,
    },
}

/// The data that is stored in the filesystem relative
/// to a key pair. The name of the key pair is used as
/// the key to this value, therefore not included here.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct KeyPairData {
    pub seed: Seed,
}

/// The seed from which a key pair
/// can be deterministically generated.
type Seed = [u8; 32];

#[derive(Debug, ThisError)]
pub enum Error {
    /// A key pair with the given name already exists
    #[error("A key pair with the given name already exists")]
    AlreadyExists(),

    /// Failed to write to the key-pairs file
    #[error("{}", io_error_message("write"))]
    FailedWrite(#[from] WritingError),

    /// Failed to read the key-pairs file
    #[error("{}", io_error_message("read"))]
    FailedRead(#[from] ReadingError),

    /// Cannot read directory
    #[error("Cannot read directory '{1}'")]
    CannotReadDirectory(#[source] IOError, PathBuf),

    /// Cannot create directory
    #[error("Cannot create directory '{1}'")]
    CannotCreateDirectory(#[source] IOError, PathBuf),

    /// Could not find a key pair with the given name
    #[error("Could not find a key pair with the given name")]
    NotFound(),
}

fn io_error_message(action: &str) -> String {
    format!(
        "Failed to {} the key-pairs file: '{}'",
        action,
        FILE.display()
    )
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

/// List all the stored key pairs.
///
/// Preemptively [init()]s the storage on disk and checks permissions.
/// It can fail from IO errors or Serde Json errors.
pub fn list() -> Result<HashMap<String, KeyPairData>, Error> {
    use {KeyStorageFile::*, VersionedFile::*};

    init()?;
    match parse_file()? {
        Unversioned(key_pairs) => Ok(key_pairs),
        Versioned(V1 { key_pairs }) => Ok(key_pairs),
    }
}

/// Add a key pair to the storage.
///
/// Fails if a key pair with the given `name` already exists.
/// It can also fail from IO and Serde Json errors.
pub fn add(name: String, data: KeyPairData) -> Result<(), Error> {
    let mut key_pairs = list()?;
    if key_pairs.contains_key(&name) {
        return Err(Error::AlreadyExists());
    }

    key_pairs.insert(name, data);
    update(key_pairs)
}

/// Get a key pair by name.
///
/// It can fail from IO and Serde Json errors, or if no such
/// key pair is found.
pub fn get(name: &str) -> Result<KeyPairData, Error> {
    list()?.get(name).map(Clone::clone).ok_or(Error::NotFound())
}

fn update(key_pairs: HashMap<String, KeyPairData>) -> Result<(), Error> {
    let data = VersionedFile::V1 { key_pairs };
    let new_content = serde_json::to_string_pretty(&data).map_err(WritingError::Serialization)?;
    std::fs::write(FILE.as_path(), new_content.as_bytes()).map_err(WritingError::IO)?;
    Ok(())
}

fn parse_file() -> Result<KeyStorageFile, Error> {
    let file = File::open(FILE.as_path()).map_err(ReadingError::IO)?;
    serde_json::from_reader(&file).map_err(|e| ReadingError::Deserialization(e).into())
}

/// Build the path to the given filename under [dir()].
fn build_path(filename: &str) -> PathBuf {
    dir().join(filename)
}

fn dir() -> PathBuf {
    BaseDirs::new()
        .unwrap()
        .data_dir()
        .join("radicle-registry-cli")
}

/// Initialize the storage on disk to be used correctly.
///   * Create the directory structure and check permissions
///   * Create and initialize the FILE where the key pairs will be stored.
fn init() -> Result<(), Error> {
    let dir = FILE.parent().unwrap();
    init_dir(&dir)?;
    init_file(&FILE)?;
    Ok(())
}

/// Initialize the given directory to be used.
/// Fails with
///   * [Error::CannotCreateDirectory] if the directory
///     does not exist and fails to be created.
///   * [Error::CannotReadDirectory] if the directory
///     does exist but cannot be read.
fn init_dir(dir: &Path) -> Result<(), Error> {
    std::fs::create_dir_all(&dir)
        .map_err(|err| Error::CannotCreateDirectory(err, dir.to_path_buf()))?;
    File::open(dir).map_err(|err| Error::CannotReadDirectory(err, dir.to_path_buf()))?;
    Ok(())
}

/// Init the key-pair storage file on disk.
///
///   * Rename the legacy `accounts.json` file to the FILE name.
///   * Create FILE if it doesn't yet exist.
fn init_file(path: &Path) -> Result<(), Error> {
    if !path.exists() {
        let old_path = build_path("accounts.json");
        if old_path.exists() {
            std::fs::rename(old_path, path).map_err(WritingError::IO)?;
        } else {
            update(HashMap::new())?;
        }
    }

    Ok(())
}
