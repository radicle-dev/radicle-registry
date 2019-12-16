/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern checkpoint creation.
use futures01::prelude::*;

use radicle_registry_client::*;

mod common;

#[test]
fn create_checkpoint() {
    let client = Client::new_emulator();
    let bob = common::key_pair_from_string("Bob");

    let project_hash1 = H256::random();
    let checkpoint_id1 = common::submit_ok(
        &client,
        &bob,
        CreateCheckpointParams {
            project_hash: project_hash1,
            previous_checkpoint_id: None,
        },
    )
    .result
    .unwrap();

    let project_hash2 = H256::random();
    let checkpoint_id2 = common::submit_ok(
        &client,
        &bob,
        CreateCheckpointParams {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(checkpoint_id1),
        },
    )
    .result
    .unwrap();

    let checkpoint1_ = Checkpoint {
        parent: None,
        hash: project_hash1,
    };
    let checkpoint1 = client
        .get_checkpoint(checkpoint_id1)
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(checkpoint1, checkpoint1_);

    let checkpoint2_ = Checkpoint {
        parent: Some(checkpoint_id1),
        hash: project_hash2,
    };
    let checkpoint2 = client
        .get_checkpoint(checkpoint_id2)
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(checkpoint2, checkpoint2_);
}

#[test]
fn create_checkpoint_without_parent() {
    let client = Client::new_emulator();
    let bob = common::key_pair_from_string("Bob");

    let project_hash = H256::random();
    let previous_checkpoint_id = Some(CheckpointId::random());

    let tx_applied = common::submit_ok(
        &client,
        &bob,
        CreateCheckpointParams {
            project_hash,
            previous_checkpoint_id,
        },
    );

    assert_eq!(tx_applied.result, Err(DispatchError::Other("")))
}
