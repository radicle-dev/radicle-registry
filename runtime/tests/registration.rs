/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern project registration.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn register_project() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let project_hash = H256::random();
    let checkpoint_id = submit_ok(
        &client,
        &alice,
        messages::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id: None,
        },
    )
    .await
    .result
    .unwrap();
    let message = random_register_project_message(checkpoint_id);
    let tx_applied = submit_ok(&client, &alice, message.clone()).await;

    let project = client
        .get_project(message.clone().id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(project.id, message.clone().id);
    assert_eq!(project.current_cp, checkpoint_id);

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::ProjectRegistered(message.clone().id, project.account_id).into()
    );

    let has_project = client
        .list_projects()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == message.id);
    assert!(has_project, "Registered project not found in project list");

    let checkpoint_ = Checkpoint {
        parent: None,
        hash: project_hash,
    };
    let checkpoint = client.get_checkpoint(checkpoint_id).await.unwrap().unwrap();
    assert_eq!(checkpoint, checkpoint_);
}

#[async_std::test]
async fn register_project_with_duplicate_id() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let checkpoint_id = submit_ok(
        &client,
        &alice,
        messages::CreateCheckpoint {
            project_hash: H256::random(),
            previous_checkpoint_id: None,
        },
    )
    .await
    .result
    .unwrap();

    let message = random_register_project_message(checkpoint_id);

    submit_ok(&client, &alice, message.clone()).await;

    // Duplicate submission with different description and image URL.
    let registration_2 = submit_ok(
        &client,
        &alice,
        messages::RegisterProject { ..message.clone() },
    )
    .await;

    assert_eq!(registration_2.result, Err(DispatchError::Other("")));

    let _project = client.get_project(message.id).await.unwrap().unwrap();

    // TODO(nuno): assert that the project data is left untouched.
    // This can be done again when the metadata field is added.
}

#[async_std::test]
async fn register_project_with_bad_checkpoint() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let checkpoint_id = H256::random();

    let message = random_register_project_message(checkpoint_id);

    let tx_applied = submit_ok(&client, &alice, message.clone()).await;

    assert_eq!(tx_applied.result, Err(DispatchError::Other("")));

    let no_project = client.get_project(message.id).await.unwrap();

    assert!(no_project.is_none())
}
