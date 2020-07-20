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
use radicle_registry_core::state;

/// Submit a transaction and wait for it to be successfully applied.
///
/// Panics if submission errors.
pub async fn submit_ok_with_fee<Message_: Message>(
    client: &Client,
    author: &ed25519::Pair,
    message: Message_,
    fee: Balance,
) -> TransactionIncluded<Message_> {
    client
        .sign_and_submit_message(&author, message, fee)
        .await
        .unwrap()
        .await
        .unwrap()
}

/// Submit a transaction and wait for it to be successfully applied.
///
/// Panics if submission errors.
pub async fn submit_ok<Message_: Message>(
    client: &Client,
    author: &ed25519::Pair,
    message: Message_,
) -> TransactionIncluded<Message_> {
    submit_ok_with_fee(&client, &author, message, random_balance()).await
}

pub async fn create_project(
    client: &Client,
    author: &ed25519::Pair,
    domain: &ProjectDomain,
) -> (ProjectName, state::Projects1Data) {
    let register_project_message = random_register_project_message(domain);
    submit_ok(&client, &author, register_project_message.clone()).await;
    let project = client
        .get_project(
            register_project_message.project_name.clone(),
            domain.clone(),
        )
        .await
        .unwrap()
        .unwrap();
    (register_project_message.project_name, project)
}

pub fn random_id() -> Id {
    let size = rand::thread_rng().gen_range(1, 33);
    Id::try_from(random_alnum_string(size).to_lowercase()).unwrap()
}

pub fn random_project_name() -> ProjectName {
    let size = rand::thread_rng().gen_range(1, 33);
    ProjectName::try_from(random_alnum_string(size).to_lowercase()).unwrap()
}

/// Create a [message::RegisterOrg] with random parameters.
pub fn random_register_org_message() -> message::RegisterOrg {
    message::RegisterOrg {
        org_id: random_id(),
    }
}

/// Create a [message::RegisterProject] with random parameters to register a project with.
pub fn random_register_project_message(domain: &ProjectDomain) -> message::RegisterProject {
    message::RegisterProject {
        project_name: random_project_name(),
        project_domain: domain.clone(),
        metadata: Bytes128::random(),
    }
}

/// Create a [message::RegisterUser] with random parameters.
pub fn random_register_user_message() -> message::RegisterUser {
    message::RegisterUser {
        user_id: random_id(),
    }
}

pub fn root_key_pair() -> ed25519::Pair {
    ed25519::Pair::from_string("//Alice", None).unwrap()
}

/// Generate a random a key pair and equip the account with some funds.
pub async fn key_pair_with_funds(client: &Client) -> ed25519::Pair {
    let key_pair = ed25519::Pair::generate().0;

    transfer(&client, &root_key_pair(), key_pair.public(), 100_000).await;

    key_pair
}

/// Create a random key pair derived and register a user associated with it.
/// Ensures that the account for the key pair is equipped with enough RAD to run transactions.
pub async fn key_pair_with_associated_user(client: &Client) -> (ed25519::Pair, Id) {
    let key_pair = key_pair_with_funds(&client).await;
    let user_id = associate_key_pair_with_random_user(client, &key_pair).await;

    (key_pair, user_id)
}

/// Register a User associated with the given key pair. Returns the new, associated user Id.
pub async fn associate_key_pair_with_random_user(client: &Client, key_pair: &ed25519::Pair) -> Id {
    let user_id = random_id();
    let register_user_message = message::RegisterUser {
        user_id: user_id.clone(),
    };
    let tx_applied = submit_ok(&client, &key_pair, register_user_message).await;
    assert_eq!(tx_applied.result, Ok(()));

    user_id
}

pub fn random_alnum_string(size: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .collect::<String>()
}

/// Check if the user with the given id exists in the chain state.
pub async fn user_exists(client: &Client, user_id: Id) -> bool {
    client
        .list_users()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == user_id.clone())
}

pub fn random_balance() -> Balance {
    rand::thread_rng().gen_range(20, 100)
}

pub async fn transfer(
    client: &Client,
    donator: &ed25519::Pair,
    recipient: AccountId,
    amount: Balance,
) {
    let tx_included = submit_ok_with_fee(
        &client,
        &donator,
        message::Transfer { recipient, amount },
        1,
    )
    .await;
    assert_eq!(
        tx_included.result,
        Ok(()),
        "Failed to grant funds to the recipient account."
    );
}

/// Generate project domains owned by the given `author`. It associates the author
/// with a random user and registers a random org with the author account.
pub async fn generate_project_domains(
    client: &Client,
    author: &ed25519::Pair,
) -> Vec<ProjectDomain> {
    let user_id = associate_key_pair_with_random_user(client, author).await;
    let (org_id, _) = register_random_org(&client, &author).await;

    vec![ProjectDomain::User(user_id), ProjectDomain::Org(org_id)]
}

/// Register a random org with the given author that becomes its only member.
/// Equips the key pair account with enough funds to run transactions.
pub async fn register_random_org(
    client: &Client,
    author: &ed25519::Pair,
) -> (Id, state::Orgs1Data) {
    let register_org = random_register_org_message();
    let org_id = register_org.org_id.clone();
    submit_ok(&client, author, register_org).await;

    let org = client.get_org(org_id.clone()).await.unwrap().unwrap();
    transfer(&client, &author, org.account_id(), 1000).await;

    (org_id, org)
}
