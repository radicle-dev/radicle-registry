/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
use futures::prelude::*;

use radicle_registry_client::*;

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
    assert_eq!(project.id, project_id);
    assert_eq!(project.description, "DESCRIPTION");
    assert_eq!(project.img_url, "IMG_URL");
    assert_eq!(project.current_cp, checkpoint_id);

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::ProjectRegistered(project_id.clone(), project.account_id).into()
    );

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
fn register_project_with_duplicate_id() {
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

    let params = RegisterProjectParams {
        id: project_id.clone(),
        description: "DESCRIPTION".to_string(),
        img_url: "IMG_URL".to_string(),
        checkpoint_id,
    };

    client
        .register_project(&alice, params.clone())
        .wait()
        .unwrap();

    // Duplicate submission with different description and image URL.
    let registration_2 = client
        .submit(
            &alice,
            RegisterProjectParams {
                description: "DESCRIPTION_2".to_string(),
                img_url: "IMG_URL_2".to_string(),
                ..params
            },
        )
        .wait()
        .unwrap();

    assert_eq!(registration_2.result, Err(None));

    let project = client.get_project(project_id).wait().unwrap().unwrap();

    assert_eq!("DESCRIPTION", project.description);
    assert_eq!("IMG_URL", project.img_url)
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
fn create_checkpoint_without_parent() {
    let client = MemoryClient::new();
    let bob = key_pair_from_string("Bob");

    let checkpoint_id = CheckpointId::random();
    let project_hash = H256::random();
    let previous_checkpoint_id = Some(CheckpointId::random());

    let tx_applied = client
        .submit(
            &bob,
            CreateCheckpointParams {
                checkpoint_id,
                project_hash,
                previous_checkpoint_id,
            },
        )
        .wait()
        .unwrap();

    assert_eq!(tx_applied.result, Err(None))
}

#[test]
fn set_checkpoint() {
    let client = MemoryClient::new();
    let charles = key_pair_from_string("Charles");

    let project = create_project_with_checkpoint(&client, &charles);

    let project_hash2 = H256::random();
    let new_checkpoint_id = client
        .create_checkpoint(&charles, project_hash2, Some(project.current_cp))
        .wait()
        .unwrap();

    client
        .set_checkpoint(
            &charles,
            SetCheckpointParams {
                project_id: project.id.clone(),
                new_checkpoint_id,
            },
        )
        .wait()
        .unwrap();

    let new_project = client.get_project(project.id).wait().unwrap().unwrap();
    assert_eq!(new_checkpoint_id, new_project.current_cp)
}

#[test]
fn set_checkpoint_without_permission() {
    let client = MemoryClient::new();
    let eve = key_pair_from_string("Eve");

    let project = create_project_with_checkpoint(&client, &eve);

    let project_hash2 = H256::random();
    let new_checkpoint_id = client
        .create_checkpoint(&eve, project_hash2, Some(project.current_cp))
        .wait()
        .unwrap();

    let frank = key_pair_from_string("Frank");
    let tx_applied = client
        .submit(
            &frank,
            SetCheckpointParams {
                project_id: project.id.clone(),
                new_checkpoint_id,
            },
        )
        .wait()
        .unwrap();

    let updated_project = client
        .get_project(project.id.clone())
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(tx_applied.result, Err(None));
    assert_eq!(updated_project.current_cp, project.current_cp);
    assert_ne!(updated_project.current_cp, new_checkpoint_id);
}

#[test]
fn fail_to_set_nonexistent_checkpoint() {
    let client = MemoryClient::new();
    let david = key_pair_from_string("David");

    let project = create_project_with_checkpoint(&client, &david);

    let garbage = CheckpointId::random();

    let tx_applied = client
        .submit(
            &david,
            SetCheckpointParams {
                project_id: project.id.clone(),
                new_checkpoint_id: garbage,
            },
        )
        .wait()
        .unwrap();

    assert_eq!(tx_applied.result, Err(None));
    let updated_project = client
        .get_project(project.id.clone())
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(updated_project.current_cp, project.current_cp);
    assert_ne!(updated_project.current_cp, garbage);
}

#[test]
fn transfer_fail() {
    let client = MemoryClient::new();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob").public();

    let balance_alice = client.free_balance(&alice.public()).wait().unwrap();
    let tx_applied = client
        .submit(
            &alice,
            TransferParams {
                recipient: bob,
                balance: balance_alice + 1,
            },
        )
        .wait()
        .unwrap();
    assert_eq!(tx_applied.result, Err(None));
}

/// Test that we can transfer money to a project and that the project owner can transfer money from
/// a project to another account.
#[test]
fn project_account_transfer() {
    let client = MemoryClient::new();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob").public();
    let project = create_project_with_checkpoint(&client, &alice);

    assert_eq!(client.free_balance(&project.account_id).wait().unwrap(), 0);
    client
        .transfer(&alice, &project.account_id, 2000)
        .wait()
        .unwrap();
    assert_eq!(
        client.free_balance(&project.account_id).wait().unwrap(),
        2000
    );

    assert_eq!(client.free_balance(&bob).wait().unwrap(), 0);

    client
        .submit(
            &alice,
            TransferFromProjectParams {
                project: project.id.clone(),
                recipient: bob,
                value: 1000,
            },
        )
        .wait()
        .unwrap();
    assert_eq!(client.free_balance(&bob).wait().unwrap(), 1000);
    assert_eq!(
        client.free_balance(&project.account_id).wait().unwrap(),
        1000
    );
}

#[test]
/// Test that a transfer from a project account fails if the sender is not a project member.
fn project_account_transfer_non_member() {
    let client = MemoryClient::new();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob");
    let project = create_project_with_checkpoint(&client, &alice);

    client
        .transfer(&alice, &project.account_id, 2000)
        .wait()
        .unwrap();
    assert_eq!(
        client.free_balance(&project.account_id).wait().unwrap(),
        2000
    );

    client
        .submit(
            &bob,
            TransferFromProjectParams {
                project: project.id.clone(),
                recipient: bob.public(),
                value: 1000,
            },
        )
        .wait()
        .unwrap();

    assert_eq!(
        client.free_balance(&project.account_id).wait().unwrap(),
        2000
    );
}

fn create_project_with_checkpoint(client: &MemoryClient, author: &ed25519::Pair) -> Project {
    let project_id = ("NAME".parse().unwrap(), "DOMAIN".parse().unwrap());

    let checkpoint_id = client
        .create_checkpoint(&author, H256::random(), None)
        .wait()
        .unwrap();

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

    client.get_project(project_id).wait().unwrap().unwrap()
}

fn key_pair_from_string(value: impl AsRef<str>) -> ed25519::Pair {
    ed25519::Pair::from_string(format!("//{}", value.as_ref()).as_str(), None).unwrap()
}
