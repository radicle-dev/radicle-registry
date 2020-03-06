/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern checkpoint creation and setting project
/// checkpoints.
use radicle_registry_client::*;
use radicle_registry_runtime::fees::{BaseFee, Fee};
use radicle_registry_test_utils::*;

#[async_std::test]
/// Test the SetCheckpoint and that the tx fees are correctly withdrawn.
async fn set_checkpoint() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let org_id = random_string32();
    let project = create_project_with_checkpoint(org_id.clone(), &client, &alice).await;
    let org = client
        .get_org(project.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    // The org needs funds to pay for the fees involving it.
    grant_funds(&client, &alice, org.account_id, 1000).await;
    let project_name = project.clone().name;

    let project_hash2 = H256::random();
    let new_checkpoint_id = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(project.current_cp),
            bid: 10,
        },
    )
    .await
    .result
    .unwrap();

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();

    let bid = random_balance();
    submit_ok(
        &client,
        &alice,
        message::SetCheckpoint {
            project_name: project.name,
            org_id: project.org_id,
            new_checkpoint_id,
            bid,
        },
    )
    .await;

    let new_project = client
        .get_project(project_name, org_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(new_checkpoint_id, new_project.current_cp);

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "Tx author should have paid for all fees"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before - (bid - BaseFee.value()),
        "The org should have paid for the tip"
    );
}

#[async_std::test]
async fn set_checkpoint_without_permission() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let org_id = random_string32();
    let project = create_project_with_checkpoint(org_id.clone(), &client, &alice).await;
    let project_name = project.name.clone();

    let project_hash2 = H256::random();
    let new_checkpoint_id = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(project.current_cp),
            bid: 10,
        },
    )
    .await
    .result
    .unwrap();

    let bad_actor = key_pair_from_string("BadActor");
    // The bad actor needs some balance to run transactions.
    grant_funds(&client, &alice, bad_actor.public(), 1000).await;
    let bad_actor_balance_before = client.free_balance(&bad_actor.public()).await.unwrap();

    let bid = random_balance();
    let tx_applied = submit_ok(
        &client,
        &bad_actor,
        message::SetCheckpoint {
            project_name: project.name,
            org_id: project.org_id,
            new_checkpoint_id,
            bid,
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

    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        bad_actor_balance_before - BaseFee.value(),
        "Tx author should have (only) paid for the base fee"
    );
}

#[async_std::test]
async fn fail_to_set_nonexistent_checkpoint() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let org_id = random_string32();
    let project = create_project_with_checkpoint(org_id.clone(), &client, &alice).await;
    let org = client
        .get_org(project.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    let project_name = project.name.clone();

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();

    let garbage = CheckpointId::random();
    let bid = random_balance();
    let tx_applied = submit_ok(
        &client,
        &alice,
        message::SetCheckpoint {
            project_name: project.name,
            org_id: project.org_id,
            new_checkpoint_id: garbage,
            bid,
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

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "Tx author should have (only) paid the base fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before - (bid - BaseFee.value()),
        "The org should have (only) paid for the tip"
    )
}

#[async_std::test]
async fn set_fork_checkpoint() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let org_id = random_string32();
    let project = create_project_with_checkpoint(org_id.clone(), &client, &alice).await;
    let project_name = project.name.clone();
    let mut current_cp = project.current_cp;

    // How many checkpoints to create.
    let n = 5;
    let mut checkpoints: Vec<CheckpointId> = Vec::with_capacity(n);
    for _ in 0..n {
        let new_checkpoint_id = submit_ok(
            &client,
            &alice,
            message::CreateCheckpoint {
                project_hash: H256::random(),
                previous_checkpoint_id: (Some(current_cp)),
                bid: 10,
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
        &alice,
        message::CreateCheckpoint {
            project_hash: H256::random(),
            previous_checkpoint_id: (Some(checkpoints[2])),
            bid: 10,
        },
    )
    .await
    .result
    .unwrap();

    submit_ok(
        &client,
        &alice,
        message::SetCheckpoint {
            project_name: project.name,
            org_id: project.org_id,
            new_checkpoint_id: forked_checkpoint_id,
            bid: 10,
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

#[async_std::test]
async fn set_checkpoint_insufficient_funds() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let org_id = random_string32();
    let project = create_project_with_checkpoint(org_id.clone(), &client, &alice).await;

    let poor_actor = key_pair_from_string("Poor");
    assert_eq!(client.free_balance(&poor_actor.public()).await.unwrap(), 0);

    let bid = random_balance();
    let tx_applied = submit_ok(
        &client,
        &poor_actor,
        message::SetCheckpoint {
            project_name: project.name,
            org_id: project.org_id,
            new_checkpoint_id: H256::random(),
            bid,
        },
    )
    .await;

    assert_eq!(
        tx_applied.result,
        Err(RegistryError::FailedFeePayment.into())
    );
    assert_eq!(
        client.free_balance(&poor_actor.public()).await.unwrap(),
        0,
        "The tx author should have had no funds to run the tx"
    );
}
