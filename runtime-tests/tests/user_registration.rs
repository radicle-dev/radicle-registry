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
/// The tests in this module concern user registration.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn register_user() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let initial_balance = client.free_balance(&alice.public()).await.unwrap();

    let register_user_message = random_register_user_message();
    let random_fee = random_balance();
    let included_tx =
        submit_ok_with_fee(&client, &alice, register_user_message.clone(), random_fee).await;

    assert!(included_tx
        .events
        .contains(&RegistryEvent::UserRegistered(register_user_message.user_id.clone()).into()));

    assert!(
        user_exists(&client, register_user_message.user_id.clone()).await,
        "User not found in users list",
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );

    let user: User = client
        .get_user(register_user_message.user_id.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.id, register_user_message.user_id);
    assert!(user.projects.is_empty());
}

#[async_std::test]
async fn register_user_with_duplicate_id() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_user_message = random_register_user_message();

    let included_tx_once = submit_ok(&client, &alice, register_user_message.clone()).await;
    assert!(included_tx_once.result.is_ok());

    let included_tx_twice = submit_ok(&client, &alice, register_user_message.clone()).await;
    assert_eq!(
        included_tx_twice.result,
        Err(RegistryError::DuplicateUserId.into())
    )
}

#[async_std::test]
async fn register_user_with_already_associated_account() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_first_user_message = random_register_user_message();

    let included_tx_first = submit_ok(&client, &alice, register_first_user_message.clone()).await;
    assert!(included_tx_first.result.is_ok());

    // Register a different user with the same account.
    let register_second_user_message = random_register_user_message();
    let included_tx_twice = submit_ok(&client, &alice, register_second_user_message.clone()).await;
    assert_eq!(
        included_tx_twice.result,
        Err(RegistryError::UserAccountAssociated.into())
    )
}

#[async_std::test]
async fn unregister_user() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_user_message = random_register_user_message();

    // Registration.
    let included_tx = submit_ok(&client, &alice, register_user_message.clone()).await;
    assert!(included_tx
        .events
        .contains(&RegistryEvent::UserRegistered(register_user_message.user_id.clone()).into()));
    assert!(included_tx.result.is_ok());
    assert!(
        user_exists(&client, register_user_message.user_id.clone()).await,
        "User not found in users list"
    );

    // Unregistration.
    let initial_balance = client.free_balance(&alice.public()).await.unwrap();

    let unregister_user_message = message::UnregisterUser {
        user_id: register_user_message.user_id.clone(),
    };
    let random_fee = random_balance();
    let tx_unregister_applied =
        submit_ok_with_fee(&client, &alice, unregister_user_message.clone(), random_fee).await;
    assert!(tx_unregister_applied.result.is_ok());
    assert!(
        !user_exists(&client, register_user_message.user_id.clone()).await,
        "The user was not expected to exist"
    );
    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn unregister_user_with_invalid_sender() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_user_message = random_register_user_message();

    // Reistration.
    let included_tx = submit_ok(&client, &alice, register_user_message.clone()).await;
    assert!(included_tx
        .events
        .contains(&RegistryEvent::UserRegistered(register_user_message.user_id.clone()).into()));
    assert!(included_tx.result.is_ok());
    assert!(
        user_exists(&client, register_user_message.user_id.clone()).await,
        "User not found in users list",
    );

    // Invalid unregistration.
    let bad_actor = key_pair_from_string("BadActor");
    // The bad actor needs funds to submit transactions.
    transfer(&client, &alice, bad_actor.public(), 1000).await;

    let unregister_user_message = message::UnregisterUser {
        user_id: register_user_message.user_id.clone(),
    };
    let tx_unregister_applied =
        submit_ok(&client, &bad_actor, unregister_user_message.clone()).await;
    assert!(tx_unregister_applied.result.is_err());
    assert!(
        user_exists(&client, register_user_message.user_id.clone()).await,
        "The user was expected to exist"
    );
}
