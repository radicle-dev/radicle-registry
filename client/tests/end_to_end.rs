//! Test the client against a running node.
//!
//! Note that chain state is shared between the test runs.

use radicle_registry_client::{ed25519, CryptoPair, RegisterProjectParams, SyncClient};

#[test]
fn register_project() {
    env_logger::init();
    let client = SyncClient::create().unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let project_id = client
        .register_project(
            &alice,
            RegisterProjectParams {
                name: "NAME".to_string(),
                description: "DESCRIPTION".to_string(),
                img_url: "IMG_URL".to_string(),
            },
        )
        .unwrap();

    let project = client.get_project(project_id).unwrap().unwrap();
    assert_eq!(project.name, "NAME");
    assert_eq!(project.description, "DESCRIPTION");
    assert_eq!(project.img_url, "IMG_URL");

    let has_project = client
        .list_projects()
        .unwrap()
        .iter()
        .find(|id| **id == project_id)
        .is_some();
    assert!(has_project, "Registered project not found in project list")
}
