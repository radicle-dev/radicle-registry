/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern transferring funds.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn transfer_fail() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob").public();

    let balance_alice = client.free_balance(&alice.public()).await.unwrap();
    let tx_applied = submit_ok(
        &client,
        &alice,
        messages::Transfer {
            recipient: bob,
            balance: balance_alice + 1,
        },
    )
    .await;
    assert!(tx_applied.result.is_err());
}

/// Test that we can transfer money to a project and that the project owner can transfer money from
/// a project to another account.
#[async_std::test]
async fn project_account_transfer() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob").public();
    let project = create_project_with_checkpoint(&client, &alice).await;

    assert_eq!(client.free_balance(&project.account_id).await.unwrap(), 0);
    submit_ok(
        &client,
        &alice,
        messages::Transfer {
            recipient: project.account_id,
            balance: 2000,
        },
    )
    .await;
    assert_eq!(
        client.free_balance(&project.account_id).await.unwrap(),
        2000
    );

    assert_eq!(client.free_balance(&bob).await.unwrap(), 0);

    submit_ok(
        &client,
        &alice,
        messages::TransferFromProject {
            project: project.id.clone(),
            recipient: bob,
            value: 1000,
        },
    )
    .await;
    assert_eq!(client.free_balance(&bob).await.unwrap(), 1000);
    assert_eq!(
        client.free_balance(&project.account_id).await.unwrap(),
        1000
    );
}

#[async_std::test]
/// Test that a transfer from a project account fails if the sender is not a project member.
async fn project_account_transfer_non_member() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob");
    let project = create_project_with_checkpoint(&client, &alice).await;

    submit_ok(
        &client,
        &alice,
        messages::Transfer {
            recipient: project.account_id,
            balance: 2000,
        },
    )
    .await;
    assert_eq!(
        client.free_balance(&project.account_id).await.unwrap(),
        2000
    );

    submit_ok(
        &client,
        &bob,
        messages::TransferFromProject {
            project: project.id.clone(),
            recipient: bob.public(),
            value: 1000,
        },
    )
    .await;

    assert_eq!(
        client.free_balance(&project.account_id).await.unwrap(),
        2000
    );
}
