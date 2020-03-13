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

use std::convert::TryFrom;

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

pub async fn create_project_with_checkpoint(
    org_id: OrgId,
    client: &Client,
    author: &ed25519::Pair,
) -> Project {
    let checkpoint_id = submit_ok(
        &client,
        &author,
        message::CreateCheckpoint {
            project_hash: H256::random(),
            previous_checkpoint_id: None,
        },
    )
    .await
    .result
    .unwrap();

    let register_org_message = message::RegisterOrg {
        org_id: org_id.clone(),
    };
    submit_ok(&client, &author, register_org_message.clone()).await;

    let register_project_message = random_register_project_message(org_id, checkpoint_id);
    submit_ok(&client, &author, register_project_message.clone()).await;

    client
        .get_project(
            register_project_message.project_name,
            register_org_message.org_id,
        )
        .await
        .unwrap()
        .unwrap()
}

pub async fn create_random_org(client: &Client, author: &ed25519::Pair) -> Org {
    let register_org_message = random_register_org_message();
    submit_ok(&client, &author, register_org_message.clone()).await;

    client
        .get_org(register_org_message.org_id)
        .await
        .unwrap()
        .unwrap()
}

pub fn random_org_id() -> OrgId {
    let size = rand::thread_rng().gen_range(1, 33);
    OrgId::try_from(random_alnum_string(size).to_lowercase()).unwrap()
}

pub fn random_project_name() -> ProjectName {
    let size = rand::thread_rng().gen_range(1, 33);
    ProjectName::try_from(random_alnum_string(size).to_lowercase()).unwrap()
}

fn random_user_id() -> UserId {
    let size = rand::thread_rng().gen_range(1, 33);
    UserId::try_from(random_alnum_string(size).to_lowercase()).unwrap()
}

/// Create a [core::message::RegisterOrg] with random parameters.
pub fn random_register_org_message() -> message::RegisterOrg {
    message::RegisterOrg {
        org_id: random_org_id(),
    }
}

/// Create a [core::message::RegisterProject] with random parameters to register a project with.
pub fn random_register_project_message(
    org_id: OrgId,
    checkpoint_id: CheckpointId,
) -> message::RegisterProject {
    message::RegisterProject {
        project_name: random_project_name(),
        org_id,
        checkpoint_id,
        metadata: Bytes128::random(),
    }
}

/// Create a [`core::message::RegisterUser`] with random parameters.
pub fn random_register_user_message() -> message::RegisterUser {
    message::RegisterUser {
        user_id: random_user_id(),
    }
}

pub fn key_pair_from_string(value: impl AsRef<str>) -> ed25519::Pair {
    ed25519::Pair::from_string(format!("//{}", value.as_ref()).as_str(), None).unwrap()
}

pub fn random_string32() -> String32 {
    String32::from_string(random_alnum_string(32)).unwrap()
}

pub fn random_alnum_string(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .collect::<String>()
}

/// Check if the user with the given id exists in the chain state.
pub async fn user_exists(client: &Client, user_id: UserId) -> bool {
    client
        .list_users()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == user_id.clone())
}
