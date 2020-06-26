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

/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern orgs registration.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn register_org() {
    let (client, _) = Client::new_emulator();
    let (author, user_id) = key_pair_with_associated_user(&client).await;

    let initial_balance = client.free_balance(&author.public()).await.unwrap();
    let random_fee = random_balance();
    let register_org_message = random_register_org_message();
    let tx_included =
        submit_ok_with_fee(&client, &author, register_org_message.clone(), random_fee).await;
    assert_eq!(tx_included.result, Ok(()));

    assert!(
        org_exists(&client, register_org_message.org_id.clone()).await,
        "Org not found in orgs list"
    );

    let org: Org = client
        .get_org(register_org_message.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(org.id, register_org_message.org_id);
    assert_eq!(org.members, vec![user_id]);
    assert!(org.projects.is_empty());

    assert_eq!(
        client.free_balance(&author.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

/// Attempt to register an org using an author that does not
/// have a registered user associated to its account id.
#[async_std::test]
async fn register_org_no_user() {
    let (client, _) = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let initial_balance = client.free_balance(&alice.public()).await.unwrap();
    let random_fee = random_balance();
    let register_org_message = random_register_org_message();
    let tx_applied =
        submit_ok_with_fee(&client, &alice, register_org_message.clone(), random_fee).await;

    assert_eq!(
        tx_applied.result,
        Err(RegistryError::AuthorHasNoAssociatedUser.into())
    );
    assert!(
        !org_exists(&client, register_org_message.org_id.clone()).await,
        "Org shouldn't have been registered"
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

/// Test that an org can not be registered with an id already taken by another org.
#[async_std::test]
async fn register_with_id_taken_by_org() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let register_org_message = random_register_org_message();
    let tx_included_once = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(tx_included_once.result, Ok(()));

    let tx_included_twice = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(
        tx_included_twice.result,
        Err(RegistryError::IdAlreadyTaken.into())
    );
}

/// Test that an org can not be registered with an id already taken by a user.
#[async_std::test]
async fn register_with_taken_user_id() {
    let (client, _) = Client::new_emulator();
    let author = key_pair_from_string("Alice");
    let id = random_id();

    let register_user_message = message::RegisterUser {
        user_id: id.clone(),
    };
    let tx_included_user = submit_ok(&client, &author, register_user_message.clone()).await;
    assert_eq!(tx_included_user.result, Ok(()));

    let register_org_message = message::RegisterOrg { org_id: id };
    let tx_included_org = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(
        tx_included_org.result,
        Err(RegistryError::IdAlreadyTaken.into())
    );
}

#[async_std::test]
async fn register_with_unclaimable_id_org() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    // Register org
    let register_org_message = random_register_org_message();
    let tx_included = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(tx_included.result, Ok(()));

    // Unregister Org
    let unregister_org_message = message::UnregisterOrg {
        org_id: register_org_message.org_id.clone(),
    };
    let tx_unregister_applied = submit_ok(&client, &author, unregister_org_message.clone()).await;
    assert_eq!(tx_unregister_applied.result, Ok(()));

    // Try to re-register Org with the unregistered id
    let tx_included = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(tx_included.result, Err(RegistryError::UnclaimableId.into()));
}

#[async_std::test]
async fn register_with_unclaimable_id_user() {
    let (client, _) = Client::new_emulator();
    let (author, user_id) = key_pair_with_associated_user(&client).await;

    // Unregister user
    let unregister_user_message = message::UnregisterUser {
        user_id: user_id.clone(),
    };
    let tx_unregister_applied = submit_ok(&client, &author, unregister_user_message.clone()).await;
    assert_eq!(tx_unregister_applied.result, Ok(()));

    // Try to register an Org with the unregistered user id
    let register_org_message = message::RegisterOrg { org_id: user_id };
    let tx_included = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(tx_included.result, Err(RegistryError::UnclaimableId.into()));
}

#[async_std::test]
async fn unregister_org() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let register_org_message = random_register_org_message();
    let tx_included = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(tx_included.result, Ok(()));

    assert!(
        org_exists(&client, register_org_message.org_id.clone()).await,
        "Org not found in orgs list"
    );

    // Unregister
    let initial_balance = client.free_balance(&author.public()).await.unwrap();

    let unregister_org_message = message::UnregisterOrg {
        org_id: register_org_message.org_id.clone(),
    };
    let random_fee = random_balance();
    let tx_unregister_applied =
        submit_ok_with_fee(&client, &author, unregister_org_message.clone(), random_fee).await;
    assert_eq!(tx_unregister_applied.result, Ok(()));

    assert!(
        !org_exists(&client, register_org_message.org_id.clone()).await,
        "The org was not expected to exist"
    );

    assert_eq!(
        client.free_balance(&author.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn unregister_org_bad_actor() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let register_org_message = random_register_org_message();

    let tx_included = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(tx_included.result, Ok(()));

    assert!(
        org_exists(&client, register_org_message.org_id.clone()).await,
        "Org not found in orgs list"
    );

    // Unregister
    let unregister_org_message = message::UnregisterOrg {
        org_id: register_org_message.org_id.clone(),
    };

    let (bad_actor, _) = key_pair_with_associated_user(&client).await;
    let initial_balance = client.free_balance(&bad_actor.public()).await.unwrap();
    let random_fee = random_balance();
    let tx_unregister_applied = submit_ok_with_fee(
        &client,
        &bad_actor,
        unregister_org_message.clone(),
        random_fee,
    )
    .await;

    assert_eq!(
        tx_unregister_applied.result,
        Err(RegistryError::UnregisterableOrg.into())
    );
    assert!(
        org_exists(&client, register_org_message.org_id.clone()).await,
        "Org not found in orgs list"
    );
    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn unregister_org_with_projects() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let org_id = random_id();
    create_project_with_checkpoint(&ProjectDomain::Org(org_id.clone()), &client, &author).await;

    assert!(
        org_exists(&client, org_id.clone()).await,
        "Org not found in orgs list"
    );

    let org = client.get_org(org_id.clone()).await.unwrap().unwrap();

    assert_eq!(org.projects.len(), 1);

    // Unregister
    let unregister_org_message = message::UnregisterOrg {
        org_id: org_id.clone(),
    };
    let tx_unregister_applied = submit_ok(&client, &author, unregister_org_message.clone()).await;

    assert_eq!(
        tx_unregister_applied.result,
        Err(RegistryError::UnregisterableOrg.into())
    );
    assert!(
        org_exists(&client, org_id.clone()).await,
        "Org not found in orgs list"
    );
}

async fn org_exists(client: &Client, org_id: Id) -> bool {
    client
        .list_orgs()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == org_id.clone())
}
