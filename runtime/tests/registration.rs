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
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id: None,
        },
    )
    .await
    .result
    .unwrap();
    let message = random_register_project_message(checkpoint_id);

    let org_msg = message::RegisterOrg {
        id: message.id.0.clone(),
    };
    submit_ok(&client, &alice, org_msg.clone()).await;

    let tx_applied = submit_ok(&client, &alice, message.clone()).await;

    let project = client
        .get_project(message.clone().id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(project.clone().id(), message.id.clone());
    assert_eq!(project.current_cp.clone(), checkpoint_id);
    assert_eq!(project.metadata.clone(), message.metadata.clone());

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::ProjectRegistered(message.clone().id).into()
    );

    let has_project = client
        .list_projects()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == message.id);
    assert!(has_project, "Registered project not found in project list");

    let checkpoint_ = state::Checkpoint {
        parent: None,
        hash: project_hash,
    };
    let checkpoint = client.get_checkpoint(checkpoint_id).await.unwrap().unwrap();
    assert_eq!(checkpoint, checkpoint_);
}

#[async_std::test]
async fn register_project_with_inexistent_org() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let project_hash = H256::random();
    let checkpoint_id = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id: None,
        },
    )
    .await
    .result
    .unwrap();

    let message = random_register_project_message(checkpoint_id);
    let tx_applied = submit_ok(&client, &alice, message.clone()).await;

    assert_eq!(tx_applied.result, Err(RegistryError::InexistentOrg.into()));
}

#[async_std::test]
async fn register_project_with_duplicate_id() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let checkpoint_id = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash: H256::random(),
            previous_checkpoint_id: None,
        },
    )
    .await
    .result
    .unwrap();

    let message = random_register_project_message(checkpoint_id);
    let org_msg = message::RegisterOrg {
        id: message.id.0.clone(),
    };
    submit_ok(&client, &alice, org_msg.clone()).await;
    submit_ok(&client, &alice, message.clone()).await;

    // Duplicate submission with different description and image URL.
    let registration_2 = submit_ok(
        &client,
        &alice,
        message::RegisterProject {
            metadata: Bytes128::random(),
            ..message.clone()
        },
    )
    .await;

    assert_eq!(
        registration_2.result,
        Err(RegistryError::DuplicateProjectId.into())
    );

    let project = client.get_project(message.id).await.unwrap().unwrap();

    // Assert that the project data was not altered during the
    // attempt to re-register the already existing project.
    assert_eq!(message.metadata, project.metadata)
}

#[async_std::test]
async fn register_project_with_bad_checkpoint() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let checkpoint_id = H256::random();

    let message = random_register_project_message(checkpoint_id);

    let tx_applied = submit_ok(&client, &alice, message.clone()).await;
    let org_msg = message::RegisterOrg {
        id: message.id.0.clone(),
    };
    submit_ok(&client, &alice, org_msg.clone()).await;

    assert_eq!(
        tx_applied.result,
        Err(RegistryError::InexistentCheckpointId.into())
    );

    let no_project = client.get_project(message.id).await.unwrap();

    assert!(no_project.is_none())
}
