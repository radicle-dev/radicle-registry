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

//! Miscellaneous helpers used throughout Registry tests.

use rand::distributions::Alphanumeric;
use rand::Rng;

use radicle_registry_client::*;

/// Submit a transaction and wait for it to be successfully applied.
///
/// Panics if submission errors.
pub async fn submit_ok<Message_: Message>(
    client: &Client,
    author: &ed25519::Pair,
    message: Message_,
) -> TransactionApplied<Message_> {
    client
        .sign_and_submit_message(&author, message)
        .await
        .unwrap()
        .await
        .unwrap()
}

pub async fn create_project_with_checkpoint(client: &Client, author: &ed25519::Pair) -> Project {
    let checkpoint_id = submit_ok(
        &client,
        &author,
        messages::CreateCheckpoint {
            project_hash: H256::random(),
            previous_checkpoint_id: None,
        },
    )
    .await
    .result
    .unwrap();

    let message = random_register_project_message(checkpoint_id);

    submit_ok(&client, &author, message.clone()).await;

    client.get_project(message.id).await.unwrap().unwrap()
}

/// Create random parameters to register a project with.
/// The project's name and domain will be alphanumeric strings with 32
/// characters, and the description and image URL will be alphanumeric strings
/// with 50 characters.
pub fn random_register_project_message(checkpoint_id: CheckpointId) -> messages::RegisterProject {
    let name = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .collect::<String>();
    let domain = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .collect::<String>();
    let id = (name.parse().unwrap(), domain.parse().unwrap());

    messages::RegisterProject { id, checkpoint_id }
}

pub fn key_pair_from_string(value: impl AsRef<str>) -> ed25519::Pair {
    ed25519::Pair::from_string(format!("//{}", value.as_ref()).as_str(), None).unwrap()
}
