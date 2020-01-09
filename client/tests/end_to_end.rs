//! Test the client against a running node.
//!
//! Note that chain state is shared between the test runs.

use futures01::future::Future as _;
use radicle_registry_client::*;

mod common;

#[test]
fn register_project() {
    let _ = env_logger::try_init();
    let client = Client::create_with_executor().wait().unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let project_hash = H256::random();
    let checkpoint_id = common::submit_ok(
        &client,
        &alice,
        CreateCheckpointParams {
            project_hash,
            previous_checkpoint_id: None,
        },
    )
    .result
    .unwrap();

    let register_project_params = common::random_register_project_params(checkpoint_id);

    let project_id = register_project_params.id.clone();
    let tx_applied = common::submit_ok(&client, &alice, register_project_params.clone());

    assert_eq!(tx_applied.result, Ok(()));

    let project = client
        .get_project(project_id.clone())
        .wait()
        .unwrap()
        .unwrap();
    assert_eq!(project.id, register_project_params.id.clone());
    assert_eq!(project.description, register_project_params.description);
    assert_eq!(project.img_url, register_project_params.img_url);
    assert_eq!(project.current_cp, register_project_params.checkpoint_id);

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

#[test]
/// Submit a transaction with an invalid genesis hash and expect an error.
fn invalid_transaction() {
    let _ = env_logger::try_init();
    let client = Client::create_with_executor().wait().unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let transfer_tx = Transaction::new_signed(
        &alice,
        TransferParams {
            recipient: alice.public(),
            balance: 1000,
        },
        TransactionExtra {
            nonce: 0,
            genesis_hash: Hash::zero(),
        },
    );

    let response = client.submit_transaction(transfer_tx).wait();
    match response {
        Err(Error::Other(_)) => (),
        Err(error) => panic!("Unexpected error {:?}", error),
        Ok(_) => panic!("Transaction was accepted unexpectedly"),
    }
}
