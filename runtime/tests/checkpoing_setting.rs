/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern checkpoint creation and setting project
/// checkpoints.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn set_checkpoint() {
    let client = Client::new_emulator();
    let author = key_pair_from_string("Alice");

    let org_id = random_string32();
    let project = create_project_with_checkpoint(org_id.clone(), &client, &author).await;
    let project_name = project.clone().name;

    let initial_balance = client.free_balance(&author.public()).await.unwrap();

    let project_hash2 = H256::random();
    let new_checkpoint_id = submit_ok(
        &client,
        &author,
        message::CreateCheckpoint {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(project.current_cp),
        },
    )
    .await
    .result
    .unwrap();

    submit_ok(
        &client,
        &author,
        message::SetCheckpoint {
            project_name: project.name,
            org_id: project.org_id,
            new_checkpoint_id,
        },
    )
    .await;

    // Test that the base fee (1 RAD) + Tip of 123 were withdrew.
    assert_eq!(
        initial_balance - client.free_balance(&author.public()).await.unwrap(),
        124
    );

    let new_project = client
        .get_project(project_name, org_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(new_checkpoint_id, new_project.current_cp)
}

#[async_std::test]
async fn set_checkpoint_without_permission() {
    let client = Client::new_emulator();
    let eve = key_pair_from_string("Alice");

    let org_id = random_string32();
    let project = create_project_with_checkpoint(org_id.clone(), &client, &eve).await;
    let project_name = project.name.clone();

    let project_hash2 = H256::random();
    let new_checkpoint_id = submit_ok(
        &client,
        &eve,
        message::CreateCheckpoint {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(project.current_cp),
        },
    )
    .await
    .result
    .unwrap();

    let frank = key_pair_from_string("Frank");
    let tx_applied = submit_ok(
        &client,
        &frank,
        message::SetCheckpoint {
            project_name: project.name,
            org_id: project.org_id,
            new_checkpoint_id,
        },
    )
    .await;

    let updated_project = client
        .get_project(project_name, org_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        tx_applied.result,
        Err(RegistryError::InsufficientSenderPermissions.into())
    );
    assert_eq!(updated_project.current_cp, project.current_cp.clone());
    assert_ne!(updated_project.current_cp, new_checkpoint_id);
}

#[async_std::test]
async fn fail_to_set_nonexistent_checkpoint() {
    let client = Client::new_emulator();
    let david = key_pair_from_string("David");

    let org_id = random_string32();
    let project = create_project_with_checkpoint(org_id.clone(), &client, &david).await;
    let project_name = project.name.clone();
    let garbage = CheckpointId::random();

    let tx_applied = submit_ok(
        &client,
        &david,
        message::SetCheckpoint {
            project_name: project.name,
            org_id: project.org_id,
            new_checkpoint_id: garbage,
        },
    )
    .await;

    assert_eq!(
        tx_applied.result,
        Err(RegistryError::InexistentCheckpointId.into())
    );
    let updated_project = client
        .get_project(project_name, org_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_project.current_cp, project.current_cp);
    assert_ne!(updated_project.current_cp, garbage);
}

#[async_std::test]
async fn set_fork_checkpoint() {
    let client = Client::new_emulator();
    let grace = key_pair_from_string("Grace");

    let org_id = random_string32();
    let project = create_project_with_checkpoint(org_id.clone(), &client, &grace).await;
    let project_name = project.name.clone();
    let mut current_cp = project.current_cp;

    // How many checkpoints to create.
    let n = 5;
    let mut checkpoints: Vec<CheckpointId> = Vec::with_capacity(n);
    for _ in 0..n {
        let new_checkpoint_id = submit_ok(
            &client,
            &grace,
            message::CreateCheckpoint {
                project_hash: H256::random(),
                previous_checkpoint_id: (Some(current_cp)),
            },
        )
        .await
        .result
        .unwrap();
        current_cp = new_checkpoint_id;
        checkpoints.push(new_checkpoint_id);
    }

    let forked_checkpoint_id = submit_ok(
        &client,
        &grace,
        message::CreateCheckpoint {
            project_hash: H256::random(),
            previous_checkpoint_id: (Some(checkpoints[2])),
        },
    )
    .await
    .result
    .unwrap();

    submit_ok(
        &client,
        &grace,
        message::SetCheckpoint {
            project_name: project.name,
            org_id: project.org_id,
            new_checkpoint_id: forked_checkpoint_id,
        },
    )
    .await;

    let project_1 = client
        .get_project(project_name, org_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(project_1.current_cp, forked_checkpoint_id)
}
