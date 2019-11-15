/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
use futures::prelude::*;

use radicle_registry_memory_client::{
    ed25519, Call, Checkpoint, CheckpointId, Client, CryptoPair, MemoryClient, ProjectId,
    RegisterProjectParams, RegistryEvent, SetCheckpointParams, H256,
};
use radicle_registry_runtime::registry::{ProjectDomain, ProjectName, String32};

use std::panic;

/// Helper to register project in tests.
fn create_project_with_checkpoint(
    client: &MemoryClient,
    author: &ed25519::Pair,
) -> (ProjectId, CheckpointId) {
    let project_hash = H256::random();
    let checkpoint_id = client
        .create_checkpoint(author, project_hash, None)
        .wait()
        .unwrap();
    let project_id = (
        ProjectName::from_string("NAME".to_string()).unwrap(),
        ProjectDomain::from_string("DOMAIN".to_string()).unwrap(),
    );
    client
        .register_project(
            &author,
            RegisterProjectParams {
                id: project_id.clone(),
                description: "DESCRIPTION".to_string(),
                img_url: "IMG_URL".to_string(),
                checkpoint_id,
            },
        )
        .wait()
        .unwrap();
    (project_id, checkpoint_id)
}

#[test]
fn register_project() {
    let client = MemoryClient::new();
    let alice = key_pair_from_string("Alice");

    let project_hash = H256::random();
    let checkpoint_id = client
        .create_checkpoint(&alice, project_hash, None)
        .wait()
        .unwrap();
    let project_id = (
        ProjectName::from_string("NAME".to_string()).unwrap(),
        ProjectDomain::from_string("DOMAIN".to_string()).unwrap(),
    );
    let tx_applied = client
        .submit(
            &alice,
            Call::RegisterProject(RegisterProjectParams {
                id: project_id.clone(),
                description: "DESCRIPTION".to_string(),
                img_url: "IMG_URL".to_string(),
                checkpoint_id,
            }),
        )
        .wait()
        .unwrap();

    assert_eq!(
        tx_applied.events[1],
        RegistryEvent::ProjectRegistered(project_id.clone()).into()
    );

    let project = client
        .get_project(project_id.clone())
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(project.id, project_id);
    assert_eq!(project.description, "DESCRIPTION");
    assert_eq!(project.img_url, "IMG_URL");
    assert_eq!(project.current_cp, checkpoint_id);

    let has_project = client
        .list_projects()
        .wait()
        .unwrap()
        .iter()
        .any(|id| *id == project_id);
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
fn long_string32() {
    fn long_string(n: usize) -> Result<String32, String> {
        String32::from_string(std::iter::repeat("X").take(n).collect::<String>())
    }
    let wrong = long_string(33);
    let right = long_string(32);

    assert!(
        wrong.is_err(),
        "Error: excessively long string converted to String32"
    );
    assert!(
        right.is_ok(),
        "Error: string with acceptable length failed conversion to String32."
    )
}

#[test]
fn create_checkpoint() {
    let client = MemoryClient::new();
    let bob = key_pair_from_string("Bob");

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
fn set_checkpoint() {
    let client = MemoryClient::new();
    let charles = key_pair_from_string("Charles");

    let (project_id, checkpoint_id) = create_project_with_checkpoint(&client, &charles);

    let project_hash2 = H256::random();
    let new_checkpoint_id = client
        .create_checkpoint(&charles, project_hash2, Some(checkpoint_id))
        .wait()
        .unwrap();

    client
        .set_checkpoint(
            &charles,
            SetCheckpointParams {
                project_id: project_id.clone(),
                new_checkpoint_id,
            },
        )
        .wait()
        .unwrap();

    let new_project = client.get_project(project_id).wait().unwrap().unwrap();
    assert_eq!(new_checkpoint_id, new_project.current_cp)
}

#[test]
fn fail_to_set_checkpoint() {
    let client = MemoryClient::new();
    let david = key_pair_from_string("David");

    let (project_id, _checkpoint_id) = create_project_with_checkpoint(&client, &david);

    let project_hash2 = H256::random();
    let garbage = CheckpointId::random();
    let new_checkpoint_id = client
        .create_checkpoint(&david, project_hash2, Some(garbage))
        .wait()
        .unwrap();

    // `panic::catch_unwind` is necessary here because it is not yet possible
    // to extract errors from failing transactions yet.
    let res = panic::catch_unwind(|| {
        client
            .set_checkpoint(
                &david,
                SetCheckpointParams {
                    project_id: project_id.clone(),
                    new_checkpoint_id,
                },
            )
            .wait()
    });
    assert!(
        res.is_err(),
        "Error: Invalid checkpoint successfully set as project's checkpoint."
    )
}

fn key_pair_from_string(value: impl AsRef<str>) -> ed25519::Pair {
    ed25519::Pair::from_string(format!("//{}", value.as_ref()).as_str(), None).unwrap()
}
