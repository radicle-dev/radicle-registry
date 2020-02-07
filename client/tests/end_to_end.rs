//! Test the client against a running node.
//!
//! Note that chain state is shared between the test runs.

use async_std;
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn register_project() {
    let _ = env_logger::try_init();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let project_hash = H256::random();
    let checkpoint_id = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id: None,
        },
    )
    .await
    .result
    .unwrap();

    let register_project_message = random_register_project_message(checkpoint_id);

    let project_id = register_project_message.id.clone();
    let tx_applied = submit_ok(&client, &alice, register_project_message.clone()).await;

    assert_eq!(tx_applied.result, Ok(()));

    let project = client
        .get_project(project_id.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(project.id, register_project_message.id.clone());
    assert_eq!(project.current_cp, register_project_message.checkpoint_id);
    assert_eq!(project.metadata, register_project_message.metadata);

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::ProjectRegistered(project_id.clone(), project.account_id).into()
    );

    let checkpoint = client.get_checkpoint(checkpoint_id).await.unwrap().unwrap();
    let checkpoint_ = state::Checkpoint {
        parent: None,
        hash: project_hash,
    };
    assert_eq!(checkpoint, checkpoint_);

    let has_project = client
        .list_projects()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == project_id.clone());
    assert!(has_project, "Registered project not found in project list")
}

#[async_std::test]
/// Submit a transaction with an invalid genesis hash and expect an error.
async fn invalid_transaction() {
    let _ = env_logger::try_init();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await.unwrap();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let transfer_tx = Transaction::new_signed(
        &alice,
        message::Transfer {
            recipient: alice.public(),
            balance: 1000,
        },
        TransactionExtra {
            nonce: 0,
            genesis_hash: Hash::zero(),
        },
    );

    let response = client.submit_transaction(transfer_tx).await;
    match response {
        Err(Error::Other(_)) => (),
        Err(error) => panic!("Unexpected error {:?}", error),
        Ok(_) => panic!("Transaction was accepted unexpectedly"),
    }
}
