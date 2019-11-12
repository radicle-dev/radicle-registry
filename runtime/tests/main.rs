/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
use futures::prelude::*;

use radicle_registry_memory_client::{
    ed25519, Client, CryptoPair, MemoryClient, RegisterProjectParams,
};

#[test]
fn register_project() {
    let client = MemoryClient::new();
    let alice = key_pair_from_string("Alice");

    let project_id = ("NAME".to_string(), "DOMAIN".to_string());
    client
        .register_project(
            &alice,
            RegisterProjectParams {
                id: project_id.clone(),
                description: "DESCRIPTION".to_string(),
                img_url: "IMG_URL".to_string(),
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

    let has_project = client
        .list_projects()
        .wait()
        .unwrap()
        .iter()
        .find(|id| **id == project_id)
        .is_some();
    assert!(has_project, "Registered project not found in project list")
}

fn key_pair_from_string(value: impl AsRef<str>) -> ed25519::Pair {
    ed25519::Pair::from_string(format!("//{}", value.as_ref()).as_str(), None).unwrap()
}
