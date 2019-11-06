//! Test the client against a running node.
//!
//! Note that chain state is shared between the test runs.

use radicle_registry_client::{
    ed25519, Checkpoint, CryptoPair, RegisterProjectParams, SyncClient, H256,
};

#[test]
fn register_project() {
    env_logger::init();
    let client = SyncClient::create().unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let project_hash = H256::random();
    let checkpoint_id = client
        .create_checkpoint(&alice, project_hash, None)
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
        .unwrap();

    let project = client.get_project(project_id.clone()).unwrap().unwrap();
    assert_eq!(project.id, ("NAME".to_string(), "DOMAIN".to_string()));
    assert_eq!(project.description, "DESCRIPTION");
    assert_eq!(project.img_url, "IMG_URL");
    assert_eq!(project.current_cp, checkpoint_id);

    let checkpoint = client.get_checkpoint(checkpoint_id).unwrap().unwrap();
    let checkpoint_ = Checkpoint {
        parent: None,
        hash: project_hash,
    };
    assert_eq!(checkpoint, checkpoint_);

    let has_project = client
        .list_projects()
        .unwrap()
        .iter()
        .find(|id| **id == project_id.clone())
        .is_some();
    assert!(has_project, "Registered project not found in project list")
}
