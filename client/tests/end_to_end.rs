//! Test the client against a running node.
//!
//! Note that chain state is shared between the test runs.

use futures01::future::Future as _;
use radicle_registry_client::{
    ed25519, Checkpoint, Client, ClientT as _, CryptoPair, RegisterProjectParams, RegistryEvent,
    H256,
};

#[test]
fn register_project() {
    env_logger::init();
    let client = Client::create_with_executor().unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let project_hash = H256::random();
    println!("hash: {:?}", project_hash);
    let checkpoint_id = client
        .create_checkpoint(&alice, project_hash, None)
        .wait()
        .unwrap();
    println!("checkpoint: {:}", checkpoint_id);
    let project_id = ( "NAME".to_string().into_bytes() , "DOMAIN".to_string().into_bytes() );
    let tx_applied = client
        .submit(
            &alice,
            RegisterProjectParams {
                id: project_id.clone(),
                description: "DESCRIPTION".to_string().into_bytes(),
                img_url: "IMG_URL".to_string().into_bytes(),
                checkpoint_id,
            },
        )
        .wait()
        .unwrap();

    assert_eq!(tx_applied.result, Ok(()));

    let project = client
        .get_project(project_id.clone())
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(project.id, project_id);
    assert_eq!(project.description, "DESCRIPTION".to_string().into_bytes());
    assert_eq!(project.img_url, "IMG_URL".to_string().into_bytes());
    assert_eq!(project.current_cp, checkpoint_id);

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::ProjectRegistered(project_id.clone(), project.account_id).into()
    );

    let checkpoint = client
        .get_checkpoint(checkpoint_id)
        .wait()
        .unwrap()
        .unwrap();
    let checkpoint_ = Checkpoint {
        parent: None,
        hash: project_hash,
    };
    assert_eq!(checkpoint, checkpoint_);

    let has_project = client
        .list_projects()
        .wait()
        .unwrap()
        .iter()
        .any(|id| *id == project_id.clone());
    assert!(has_project, "Registered project not found in project list")
}
