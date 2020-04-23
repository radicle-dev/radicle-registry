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

use serial_test::serial;

use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
#[serial]
async fn register_project() {
    let _ = env_logger::try_init();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let project_hash = H256::random();
    let msg = Message::CreateCheckpoint {
        project_hash,
        previous_checkpoint_id: None,
    };
    submit_ok(&client, &alice, msg.clone())
        .await
        .result
        .unwrap();
    let checkpoint_id = Client::checkpoint_id(msg.previous_checkpoint_id, msg.project_hash);

    let org_id = random_org_id();
    let register_org_message = Message::RegisterOrg {
        org_id: org_id.clone(),
    };
    let org_registered_tx = submit_ok(&client, &alice, register_org_message.clone()).await;
    assert_eq!(org_registered_tx.result, Ok(()));

    // The org needs funds to submit transactions.
    let org = client.get_org(org_id.clone()).await.unwrap().unwrap();
    let initial_balance = 1000;
    transfer(&client, &alice, org.account_id, initial_balance).await;

    let register_project_message = random_register_project_message(org_id.clone(), checkpoint_id);
    let project_name = register_project_message.project_name.clone();
    let random_fee = random_balance();
    let tx_applied = submit_ok_with_fee(
        &client,
        &alice,
        register_project_message.clone(),
        random_fee,
    )
    .await;
    assert_eq!(tx_applied.result, Ok(()));

    let project = client
        .get_project(project_name.clone(), org_id.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(project.name.clone(), project_name.clone());
    assert_eq!(project.org_id.clone(), org_id.clone());
    assert_eq!(
        project.current_cp.clone(),
        register_project_message.checkpoint_id
    );
    assert_eq!(project.metadata.clone(), register_project_message.metadata);

    let checkpoint = client.get_checkpoint(checkpoint_id).await.unwrap().unwrap();
    let checkpoint_ = state::Checkpoint {
        parent: None,
        hash: project_hash,
    };
    assert_eq!(checkpoint, checkpoint_);

    assert!(
        client
            .get_project(project_name.clone(), org_id.clone())
            .await
            .unwrap()
            .is_some(),
        "Registered project not found in project list"
    );

    let org: Org = client.get_org(org_id.clone()).await.unwrap().unwrap();
    assert!(
        org.projects.contains(&project_name),
        format!(
            "Expected project id {} in Org {} with projects {:?}",
            project_name, org_id, org.projects
        )
    );

    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
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
    let alice = key_pair_from_string("Alice");
    let initial_balance = client.free_balance(&alice.public()).await.unwrap();

    let register_org_message = random_register_org_message();
    let random_fee = random_balance();
    let tx_applied =
        submit_ok_with_fee(&client, &alice, register_org_message.clone(), random_fee).await;
    assert_eq!(tx_applied.result, Ok(()));

    let opt_org = client
        .get_org(register_org_message.org_id.clone())
        .await
        .unwrap();
    assert!(opt_org.is_some(), "Registered org not found in orgs list");
    let org = opt_org.unwrap();
    assert_eq!(org.id, register_org_message.org_id);
    assert_eq!(org.members, vec![alice.public()]);
    assert!(org.projects.is_empty());

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
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
    // Must be distinct sender from other user registrations in tests to avoid
    // AccountUserAssociated errors.
    let sender = ed25519::Pair::from_string("//Alice", None).unwrap();

    let register_user_message = random_register_user_message();
    let tx_applied = submit_ok(&client, &sender, register_user_message.clone()).await;

    let maybe_user = client
        .get_user(register_user_message.user_id.clone())
        .await
        .unwrap();
    assert!(
        maybe_user.is_some(),
        "Registered user not found in users list"
    );
    let user = maybe_user.unwrap();
    assert_eq!(user.id, register_user_message.user_id);
    assert!(user.projects.is_empty());

    // Unregistration.
    let unregister_user_message = Message::UnregisterUser {
        user_id: register_user_message.user_id.clone(),
    };
    let tx_unregister_applied = submit_ok(&client, &sender, unregister_user_message.clone()).await;
    assert!(tx_unregister_applied.result.is_ok());
    assert!(
        !user_exists(&client, register_user_message.user_id.clone()).await,
        "The user was not expected to exist"
    );
}

#[async_std::test]
#[serial]
/// Submit a transaction with an invalid genesis hash and expect an error.
async fn invalid_transaction() {
    let _ = env_logger::try_init();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let transfer_tx = Transaction::new_signed(
        &alice,
        Message::Transfer {
            recipient: alice.public(),
            balance: 1000,
        },
        TransactionExtra {
            nonce: 0,
            genesis_hash: Hash::zero(),
            fee: 123,
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
    let tx_author = key_pair_from_string("Alice");
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
    let tx_author = key_pair_from_string("PoorActor");
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
