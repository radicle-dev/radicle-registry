/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern user registration.
use radicle_registry_client::{self as client, ClientT};
use radicle_registry_test_utils as utils;

#[async_std::test]
async fn register_user() {
    let client = client::Client::new_emulator();
    let alice = utils::key_pair_from_string("Alice");
    let register_user_message = utils::random_register_user_message();

    let tx_applied = utils::submit_ok(&client, &alice, register_user_message.clone()).await;

    assert_eq!(
        tx_applied.events[0],
        client::RegistryEvent::UserRegistered(register_user_message.user_id.clone()).into(),
    );

    assert!(
        utils::user_exists(&client, register_user_message.user_id.clone()).await,
        "User not found in users list",
    );

    let user: client::User = client
        .get_user(register_user_message.user_id.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.id, register_user_message.user_id);
    assert!(user.projects.is_empty());
}

#[async_std::test]
async fn register_user_with_duplicate_id() {
    let client = client::Client::new_emulator();
    let alice = utils::key_pair_from_string("Alica");
    let register_user_message = utils::random_register_user_message();

    let tx_applied_once = utils::submit_ok(&client, &alice, register_user_message.clone()).await;
    assert!(tx_applied_once.result.is_ok());

    let tx_applied_twice = utils::submit_ok(&client, &alice, register_user_message.clone()).await;
    assert_eq!(
        tx_applied_twice.result,
        Err(client::RegistryError::DuplicateUserId.into())
    )
}

#[async_std::test]
async fn register_user_with_already_associated_account() {
    let client = client::Client::new_emulator();
    let alice = utils::key_pair_from_string("Alica");
    let register_first_user_message = utils::random_register_user_message();

    let tx_applied_first =
        utils::submit_ok(&client, &alice, register_first_user_message.clone()).await;
    assert!(tx_applied_first.result.is_ok());

    // Register a different user with the same account.
    let register_second_user_message = utils::random_register_user_message();
    let tx_applied_twice =
        utils::submit_ok(&client, &alice, register_second_user_message.clone()).await;
    assert_eq!(
        tx_applied_twice.result,
        Err(client::RegistryError::UserAccountAssociated.into())
    )
}

#[async_std::test]
async fn unregister_user() {
    let client = client::Client::new_emulator();
    let alice = utils::key_pair_from_string("Alice");
    let register_user_message = utils::random_register_user_message();

    // Registration.
    let tx_applied = utils::submit_ok(&client, &alice, register_user_message.clone()).await;
    assert_eq!(
        tx_applied.events[0],
        client::RegistryEvent::UserRegistered(register_user_message.user_id.clone()).into()
    );
    assert!(tx_applied.result.is_ok());
    assert!(
        utils::user_exists(&client, register_user_message.user_id.clone()).await,
        "User not found in users list"
    );

    // Unregistration.
    let unregister_user_message = client::message::UnregisterUser {
        user_id: register_user_message.user_id.clone(),
    };
    let tx_unregister_applied =
        utils::submit_ok(&client, &alice, unregister_user_message.clone()).await;
    assert!(tx_unregister_applied.result.is_ok());
    assert!(
        !utils::user_exists(&client, register_user_message.user_id.clone()).await,
        "The user was not expected to exist"
    );
}

#[async_std::test]
async fn unregister_user_with_invalid_sender() {
    let client = client::Client::new_emulator();
    let alice = utils::key_pair_from_string("Alice");
    let register_user_message = utils::random_register_user_message();

    // Reistration.
    let tx_applied = utils::submit_ok(&client, &alice, register_user_message.clone()).await;
    assert_eq!(
        tx_applied.events[0],
        client::RegistryEvent::UserRegistered(register_user_message.user_id.clone()).into()
    );
    assert!(tx_applied.result.is_ok());
    assert!(
        utils::user_exists(&client, register_user_message.user_id.clone()).await,
        "User not found in users list",
    );

    // Invalid unregistration.
    let bob = utils::key_pair_from_string("Bob");
    let unregister_user_message = client::message::UnregisterUser {
        user_id: register_user_message.user_id.clone(),
    };
    let tx_unregister_applied =
        utils::submit_ok(&client, &bob, unregister_user_message.clone()).await;
    assert!(tx_unregister_applied.result.is_err());
    assert!(
        utils::user_exists(&client, register_user_message.user_id.clone()).await,
        "The user was expected to exist"
    );
}
