/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern checkpoint creation.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn create_checkpoint() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();

    let project_hash1 = H256::random();
    let bid = random_balance();
    let checkpoint_id1 = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash: project_hash1,
            previous_checkpoint_id: None,
            bid,
        },
    )
    .await
    .result
    .unwrap();

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - bid,
        "Tx author should have paid for all fees"
    );

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let project_hash2 = H256::random();
    let bid = random_balance();
    let checkpoint_id2 = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(checkpoint_id1),
            bid,
        },
    )
    .await
    .result
    .unwrap();

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - bid,
        "Tx author should have paid for all fees"
    );

    let checkpoint1_ = state::Checkpoint {
        parent: None,
        hash: project_hash1,
    };
    let checkpoint1 = client
        .get_checkpoint(checkpoint_id1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(checkpoint1, checkpoint1_);

    let checkpoint2_ = state::Checkpoint {
        parent: Some(checkpoint_id1),
        hash: project_hash2,
    };
    let checkpoint2 = client
        .get_checkpoint(checkpoint_id2)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(checkpoint2, checkpoint2_);
}

#[async_std::test]
async fn create_checkpoint_without_parent() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();

    let project_hash = H256::random();
    let previous_checkpoint_id = Some(CheckpointId::random());
    let bid = random_balance();
    let tx_applied = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id,
            bid,
        },
    )
    .await;

    assert_eq!(
        tx_applied.result,
        Err(RegistryError::InexistentCheckpointId.into())
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - bid,
        "Tx author should have paid for all fees"
    );
}

#[async_std::test]
async fn create_checkpoint_insufficient_funds() {
    let client = Client::new_emulator();
    let poor_actor = key_pair_from_string("Poor");
    let poor_actor_balance_before = client.free_balance(&poor_actor.public()).await.unwrap();
    assert_eq!(poor_actor_balance_before, 0);

    let project_hash1 = H256::random();
    let bid = random_balance();
    let tx_applied = submit_ok(
        &client,
        &poor_actor,
        message::CreateCheckpoint {
            project_hash: project_hash1,
            previous_checkpoint_id: None,
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
        "Tx author should have had no funds to run the tx."
    );
}
