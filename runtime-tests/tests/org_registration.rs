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
    let client = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let initial_balance = client.free_balance(&author.public()).await.unwrap();
    let random_fee = random_balance();
    let register_org_message = random_register_org_message();
    let tx_included =
        submit_ok_with_fee(&client, &author, register_org_message.clone(), random_fee).await;

    assert!(tx_included
        .events
        .contains(&RegistryEvent::OrgRegistered(register_org_message.org_id.clone()).into()));
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
    assert_eq!(org.members, vec![author.public()]);
    assert!(org.projects.is_empty());

    assert_eq!(
        client.free_balance(&author.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn register_with_duplicated_org_id() {
    let client = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let register_org_message = random_register_org_message();

    let tx_included_once = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(tx_included_once.result, Ok(()));

    let tx_included_twice = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(
        tx_included_twice.result,
        Err(RegistryError::DuplicateOrgId.into())
    );
}

#[async_std::test]
async fn unregister_org() {
    let client = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let register_org_message = random_register_org_message();

    let tx_included = submit_ok(&client, &author, register_org_message.clone()).await;

    assert!(tx_included
        .events
        .contains(&RegistryEvent::OrgRegistered(register_org_message.org_id.clone()).into()));
    assert_eq!(tx_included.result, Ok(()));

    assert!(
        org_exists(&client, register_org_message.org_id.clone()).await,
        "Org not found in orgs list"
    );

    // Unregister
    let initial_balance = 1000;
    let org = client
        .get_org(register_org_message.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    // The org needs funds to submit transactions.
    transfer(&client, &author, org.account_id, initial_balance).await;

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
        client.free_balance(&org.account_id).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn unregister_org_bad_actor() {
    let client = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let register_org_message = random_register_org_message();

    let tx_included = submit_ok(&client, &author, register_org_message.clone()).await;

    assert!(tx_included
        .events
        .contains(&RegistryEvent::OrgRegistered(register_org_message.org_id.clone()).into()));
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
    let client = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let org_id = random_id();
    let random_project = create_project_with_checkpoint(org_id.clone(), &client, &author).await;

    assert!(
        org_exists(&client, random_project.org_id.clone()).await,
        "Org not found in orgs list"
    );

    let org = client
        .get_org(random_project.org_id.clone())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(org.projects.len(), 1);

    // Unregister
    let unregister_org_message = message::UnregisterOrg {
        org_id: random_project.org_id.clone(),
    };
    let tx_unregister_applied = submit_ok(&client, &author, unregister_org_message.clone()).await;

    assert_eq!(
        tx_unregister_applied.result,
        Err(RegistryError::UnregisterableOrg.into())
    );
    assert!(
        org_exists(&client, random_project.org_id.clone()).await,
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
