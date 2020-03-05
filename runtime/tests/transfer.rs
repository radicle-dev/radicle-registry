/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern transferring funds.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;
use radicle_registry_runtime::fees::{Fee, BaseFee};

#[async_std::test]
async fn transfer_fail() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob").public();
    let bid = 10;

    let balance_alice = client.free_balance(&alice.public()).await.unwrap();
    let tx_applied = submit_ok(
        &client,
        &alice,
        message::Transfer {
            recipient: bob,
            balance: (balance_alice - bid) + 1,
            bid,
        },
    )
    .await;
    assert!(tx_applied.result.is_err());
}

/// Test that we can transfer money to an org account and that the
/// org owner can transfer money from an org to another account.
#[async_std::test]
async fn org_account_transfer() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob").public();
    let org = create_random_org(&client, &alice).await;

    assert_eq!(client.free_balance(&org.account_id).await.unwrap(), 0);
    submit_ok(
        &client,
        &alice,
        message::Transfer {
            recipient: org.account_id,
            balance: 2000,
            bid: 10,
        },
    )
    .await;
    assert_eq!(client.free_balance(&org.account_id).await.unwrap(), 2000);

    assert_eq!(client.free_balance(&bob).await.unwrap(), 0);

    let bid = 10;
    let tip = bid - BaseFee.value();

    submit_ok(
        &client,
        &alice,
        message::TransferFromOrg {
            org_id: org.id.clone(),
            recipient: bob,
            value: 1000,
            bid,
        },
    )
    .await;
    assert_eq!(client.free_balance(&bob).await.unwrap(), 1000);
    assert_eq!(client.free_balance(&org.account_id).await.unwrap(), 1000 - tip);
}

#[async_std::test]
/// Test that a transfer from an org account fails if the sender is not an org member.
async fn org_account_transfer_non_member() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob");
    let org = create_random_org(&client, &alice).await;

    submit_ok(
        &client,
        &alice,
        message::Transfer {
            recipient: org.account_id,
            balance: 2000,
            bid: 10,
        },
    )
    .await;
    assert_eq!(client.free_balance(&org.account_id).await.unwrap(), 2000);

    submit_ok(
        &client,
        &bob,
        message::TransferFromOrg {
            org_id: org.id.clone(),
            recipient: bob.public(),
            value: 1000,
            bid: 10,
        },
    )
    .await;

    assert_eq!(client.free_balance(&org.account_id).await.unwrap(), 2000);
}
