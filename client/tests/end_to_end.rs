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

//! Test the client against a running node.
//!
//! Note that chain state is shared between the test runs.
//! To avoid AccountUserAssociated errors, use a distinct author for each test.

use serial_test::serial;

use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
#[serial]
async fn register_project() {
    let _ = env_logger::try_init();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let author = key_pair_with_funds(&client).await;

    for domain in generate_project_domains(&client, &author).await {
        let project_hash = H256::random();
        let checkpoint_id = submit_ok(
            &client,
            &author,
            message::CreateCheckpoint {
                project_hash,
                previous_checkpoint_id: None,
            },
        )
        .await
        .result
        .unwrap();

        let initial_balance = match &domain {
            ProjectDomain::Org(org_id) => {
                let org = client.get_org(org_id.clone()).await.unwrap().unwrap();
                client.free_balance(&org.account_id()).await.unwrap()
            }
            ProjectDomain::User(user_id) => {
                let user = client.get_user(user_id.clone()).await.unwrap().unwrap();
                client.free_balance(&user.account_id()).await.unwrap()
            }
        };

        let random_fee = random_balance();
        let message = random_register_project_message(&domain, checkpoint_id);
        let project_name = message.project_name.clone();
        let tx_included = submit_ok_with_fee(&client, &author, message.clone(), random_fee).await;
        assert_eq!(tx_included.result, Ok(()));

        let project = client
            .get_project(project_name.clone(), domain.clone())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(project.current_cp().clone(), checkpoint_id);
        assert_eq!(project.metadata().clone(), message.metadata.clone());

        let has_project = client
            .list_projects()
            .await
            .unwrap()
            .iter()
            .any(|id| *id == (message.project_name.clone(), message.project_domain.clone()));
        assert!(has_project, "Registered project not found in project list");

        let checkpoint = client.get_checkpoint(checkpoint_id).await.unwrap().unwrap();
        assert_eq!(checkpoint, state::Checkpoints1Data::new(None, project_hash));

        let (projects, account_id) = match &domain {
            ProjectDomain::Org(org_id) => {
                let org = client.get_org(org_id.clone()).await.unwrap().unwrap();
                (org.projects().clone(), org.account_id())
            }
            ProjectDomain::User(user_id) => {
                let user = client.get_user(user_id.clone()).await.unwrap().unwrap();
                (user.projects().clone(), user.account_id())
            }
        };

        assert_eq!(projects, vec![project_name]);
        assert_eq!(
            client.free_balance(&account_id).await.unwrap(),
            initial_balance - random_fee,
            "The tx fee was not charged properly."
        );
    }
}

#[async_std::test]
#[serial]
async fn register_member() {
    let _ = env_logger::try_init();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let (author, author_id) = key_pair_with_associated_user(&client).await;
    let (_, user_id) = key_pair_with_associated_user(&client).await;

    let org_id = random_id();
    let org_registered_tx = submit_ok(
        &client,
        &author,
        message::RegisterOrg {
            org_id: org_id.clone(),
        },
    )
    .await;
    assert_eq!(org_registered_tx.result, Ok(()));

    // The org needs funds to submit transactions.
    let org = client.get_org(org_id.clone()).await.unwrap().unwrap();
    let initial_balance = 1000;
    transfer(&client, &author, org.account_id(), initial_balance).await;

    assert_eq!(*org.members(), vec![author_id.clone()]);

    let register_member_message = message::RegisterMember {
        org_id: org_id.clone(),
        user_id: user_id.clone(),
    };
    let random_fee = random_balance();
    let tx_applied = submit_ok_with_fee(
        &client,
        &author,
        register_member_message.clone(),
        random_fee,
    )
    .await;
    assert_eq!(tx_applied.result, Ok(()));

    let re_org = client.get_org(org_id.clone()).await.unwrap().unwrap();
    assert_eq!(re_org.members().len(), 2);
    assert!(
        re_org.members().contains(&author_id),
        format!(
            "Expected author id {} in Org {} with members {:?}",
            author_id,
            org_id,
            re_org.members()
        )
    );
    assert!(
        re_org.members().contains(&user_id),
        format!(
            "Expected user id {} in Org {} with members {:?}",
            user_id,
            org_id,
            re_org.members()
        )
    );

    assert_eq!(
        client.free_balance(&org.account_id()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
#[serial]
async fn register_org() {
    let _ = env_logger::try_init();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let (author, user_id) = key_pair_with_associated_user(&client).await;

    let initial_balance = client.free_balance(&author.public()).await.unwrap();

    let org_id = random_id();
    let random_fee = random_balance();
    let tx_included = submit_ok_with_fee(
        &client,
        &author,
        message::RegisterOrg {
            org_id: org_id.clone(),
        },
        random_fee,
    )
    .await;
    assert_eq!(tx_included.result, Ok(()));

    let opt_org = client.get_org(org_id.clone()).await.unwrap();
    assert!(opt_org.is_some(), "Registered org not found in orgs list");
    let org = opt_org.unwrap();
    assert_eq!(*org.members(), vec![user_id]);
    assert!(org.projects().is_empty());

    assert_eq!(
        client.free_balance(&author.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
#[serial]
async fn register_user() {
    let _ = env_logger::try_init();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let author = ed25519::Pair::from_string("//Alice", None).unwrap();

    let user_id = random_id();
    let tx_included = submit_ok(
        &client,
        &author,
        message::RegisterUser {
            user_id: user_id.clone(),
        },
    )
    .await;
    assert_eq!(tx_included.result, Ok(()));

    let maybe_user = client.get_user(user_id.clone()).await.unwrap();
    assert!(
        maybe_user.is_some(),
        "Registered user not found in users list"
    );
    let user = maybe_user.unwrap();
    assert!(user.projects().is_empty());
}

#[async_std::test]
#[serial]
/// Submit a transaction with an invalid genesis hash and expect an error.
async fn invalid_transaction() {
    let _ = env_logger::try_init();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
    let runtime_spec_version = client.runtime_version().await.unwrap().spec_version;

    let transfer_tx = Transaction::new_signed(
        &alice,
        message::Transfer {
            recipient: alice.public(),
            balance: 1000,
        },
        TransactionExtra {
            nonce: 0,
            genesis_hash: Hash::zero(),
            fee: 123,
            runtime_spec_version,
        },
    );

    let response = client.submit_transaction(transfer_tx).await;
    match response {
        Err(Error::Rpc(_)) => (),
        Err(error) => panic!("Unexpected error {:?}", error),
        Ok(_) => panic!("Transaction was accepted unexpectedly"),
    }
}

// Test that any message submited with an insufficient fee fails.
#[async_std::test]
#[serial]
async fn insufficient_fee() {
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let tx_author = key_pair_with_funds(&client).await;
    let insufficient_fee: Balance = 0;

    let whatever_message = random_register_org_message();
    let response = client
        .sign_and_submit_message(&tx_author, whatever_message, insufficient_fee)
        .await;

    match response {
        Err(Error::Rpc(_)) => (),
        Err(error) => panic!("Unexpected error {:?}", error),
        Ok(_) => panic!("Transaction was accepted unexpectedly"),
    }
}

// Test that any message submited by an author with insufficient
// funds to pay the tx fee fails.
#[async_std::test]
#[serial]
async fn insufficient_funds() {
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let tx_author = ed25519::Pair::generate().0;
    assert_eq!(client.free_balance(&tx_author.public()).await.unwrap(), 0);

    let whatever_message = random_register_org_message();
    let random_fee = random_balance();
    let response = client
        .sign_and_submit_message(&tx_author, whatever_message, random_fee)
        .await;

    match response {
        Err(Error::Rpc(_)) => (),
        Err(error) => panic!("Unexpected error {:?}", error),
        Ok(_) => panic!("Transaction was accepted unexpectedly"),
    }
}

/// Test that an org can not be registered with an id already taken by another org.
#[async_std::test]
#[serial]
async fn register_org_with_id_taken_by_org() {
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    if client.runtime_version().await.unwrap().spec_version < 10 {
        println!("Skipping due to the incompatible runtime version");
        return;
    }
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, _) = register_random_org(&client, &author).await;

    let register_org_message = message::RegisterOrg { org_id };
    let tx_included_twice = submit_ok(&client, &author, register_org_message).await;
    assert_eq!(
        tx_included_twice.result,
        Err(RegistryError::IdAlreadyTaken.into())
    );
}

/// Test that an org can not be registered with an id already taken by a user.
#[async_std::test]
#[serial]
async fn register_org_with_taken_user_id() {
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    if client.runtime_version().await.unwrap().spec_version < 10 {
        println!("Skipping due to the incompatible runtime version");
        return;
    }
    let (author, id) = key_pair_with_associated_user(&client).await;

    let register_org_message = message::RegisterOrg { org_id: id.clone() };
    let tx_included_org = submit_ok(&client, &author, register_org_message.clone()).await;
    assert_eq!(
        tx_included_org.result,
        Err(RegistryError::IdAlreadyTaken.into())
    );
}

/// Test that a user can not be registered with an id already taken by another user.
#[async_std::test]
#[serial]
async fn register_user_with_id_taken_by_user() {
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    if client.runtime_version().await.unwrap().spec_version < 10 {
        println!("Skipping due to the incompatible runtime version");
        return;
    }
    let (author, id) = key_pair_with_associated_user(&client).await;

    let register_user_message = message::RegisterUser { user_id: id };
    let tx_included_twice = submit_ok(&client, &author, register_user_message).await;
    assert_eq!(
        tx_included_twice.result,
        Err(RegistryError::IdAlreadyTaken.into())
    );
}

/// Test that a user can not be registered with an id already taken by an org.
#[async_std::test]
#[serial]
async fn register_user_with_id_taken_by_org() {
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    if client.runtime_version().await.unwrap().spec_version < 10 {
        println!("Skipping due to the incompatible runtime version");
        return;
    }
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, _) = register_random_org(&client, &author).await;

    let register_user_message = message::RegisterUser { user_id: org_id };
    let tx_included_user = submit_ok(&client, &author, register_user_message).await;
    assert_eq!(
        tx_included_user.result,
        Err(RegistryError::IdAlreadyTaken.into())
    );
}
