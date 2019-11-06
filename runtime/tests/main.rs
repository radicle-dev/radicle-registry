/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
use futures::prelude::*;

use radicle_registry_memory_client::{
    ed25519, Checkpoint, Client, CryptoPair, MemoryClient, RegisterProjectParams, H256,
};

#[test]
fn register_project() {
    let client = MemoryClient::new();
    let alice = key_pair_from_string("Alice");

    let project_hash = H256::random();
    let checkpoint_id = client
        .create_checkpoint(&alice, project_hash, None)
        .wait()
        .unwrap();
    let project_id = ("NAME".to_string(), "DOMAIN".to_string());
    client
        .register_project(
            &alice,
            RegisterProjectParams {
                id: project_id.clone(),
                description: "DESCRIPTION".to_string(),
                img_url: "IMG_URL".to_string(),
                checkpoint_id,
            },
        )
        .wait()
        .unwrap();

    let project = client
        .get_project(project_id.clone())
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(project.id, ("NAME".to_string(), "DOMAIN".to_string()));
    assert_eq!(project.description, "DESCRIPTION");
    assert_eq!(project.img_url, "IMG_URL");
    assert_eq!(project.current_cp, checkpoint_id);

    let has_project = client
        .list_projects()
        .wait()
        .unwrap()
        .iter()
        .find(|id| **id == project_id)
        .is_some();
    assert!(has_project, "Registered project not found in project list");

    let checkpoint_ = Checkpoint {
        parent: None,
        hash: project_hash,
    };
    let checkpoint = client
        .get_checkpoint(checkpoint_id)
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(checkpoint, checkpoint_);
}

#[test]
fn create_checkpoint() {
    let client = MemoryClient::new();
    let bob = key_pair_from_string("Alice");

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

fn key_pair_from_string(value: impl AsRef<str>) -> ed25519::Pair {
    ed25519::Pair::from_string(format!("//{}", value.as_ref()).as_str(), None).unwrap()
}
