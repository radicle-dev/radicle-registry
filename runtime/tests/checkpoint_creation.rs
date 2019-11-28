/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern checkpoint creation.
use futures::prelude::*;

use radicle_registry_client::*;

mod common;

#[test]
fn create_checkpoint() {
    let client = Client::new_emulator();
    let bob = common::key_pair_from_string("Bob");

    let project_hash1 = H256::random();
    let checkpoint_id1 = client
        .create_checkpoint(&bob, project_hash1, None)
        .wait()
        .unwrap();

    let project_hash2 = H256::random();
    let checkpoint_id2 = client
        .create_checkpoint(&bob, project_hash2, Some(checkpoint_id1))
        .wait()
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

    let tx_applied = client
        .submit(
            &bob,
            CreateCheckpointParams {
                project_hash,
                previous_checkpoint_id,
            },
        )
        .wait()
        .unwrap();

    assert_eq!(tx_applied.result, Err(None))
}

#[test]
fn create_checkpoint_with_duplicate_hash() {
    let client = Client::new_emulator();
    let alice = common::key_pair_from_string("Alice");

    let project = common::create_project_with_checkpoint(&client, &alice);

    let mut current_cp = project.current_cp;

    // How many checkpoints to create.
    let n = 5;
    let mut checkpoints: Vec<CheckpointId> = Vec::with_capacity(n);
    let mut hashes: Vec<H256> = Vec::with_capacity(n);
    for _ in 0..n {
        let hash = H256::random();
        let new_checkpoint_id = client
            .create_checkpoint(&alice, hash, Some(current_cp))
            .wait()
            .unwrap();
        hashes.push(hash);
        current_cp = new_checkpoint_id;
        checkpoints.push(new_checkpoint_id);
    }

    let tx_applied = client
        .submit(
            &alice,
            CreateCheckpointParams {
                project_hash: hashes[2],
                previous_checkpoint_id: Some(checkpoints[4]),
            },
        )
        .wait()
        .unwrap();

    assert_eq!(tx_applied.result, Err(None));

    let new_checkpoint_id = client
        .create_checkpoint(&alice, hashes[4], Some(checkpoints[2]))
        .wait()
        .unwrap();

    let new_checkpoint = client
        .get_checkpoint(new_checkpoint_id)
        .wait()
        .unwrap()
        .unwrap();

    assert_eq!(new_checkpoint.hash, hashes[4])
}
