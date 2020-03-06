/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern transferring funds.
use radicle_registry_client::*;
use radicle_registry_runtime::fees::{BaseFee, Fee};
use radicle_registry_test_utils::*;

#[async_std::test]
async fn transfer_ok() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob");
    let bid = 10;

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let bob_balance_before = client.free_balance(&bob.public()).await.unwrap();
    assert_eq!(bob_balance_before, 0);

    let transfer_amount = 1000;
    let tx_applied = submit_ok(
        &client,
        &alice,
        message::Transfer {
            recipient: bob.public(),
            balance: transfer_amount,
            bid,
        },
    )
    .await;
    assert!(tx_applied.result.is_ok());

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - (transfer_amount + bid),
        "Tx author should have paid for all fees"
    );
    assert_eq!(
        client.free_balance(&bob.public()).await.unwrap(),
        transfer_amount,
    )
}
#[async_std::test]
async fn transfer_amount_exceeds_funds() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob").public();
    let bid = 10;

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let tx_applied = submit_ok(
        &client,
        &alice,
        message::Transfer {
            recipient: bob,
            balance: (alice_balance_before - bid) + 1,
            bid,
        },
    )
    .await;
    assert!(tx_applied.result.is_err());

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - bid,
        "Tx author should have paid for all fees"
    );
}

#[async_std::test]
async fn transfer_insufficient_funds() {
    let client = Client::new_emulator();
    let poor_actor = key_pair_from_string("Poor");
    let bob = key_pair_from_string("Bob").public();

    let bid = random_balance();
    assert_eq!(client.free_balance(&poor_actor.public()).await.unwrap(), 0,);
    let tx_applied = submit_ok(
        &client,
        &poor_actor,
        message::Transfer {
            recipient: bob,
            balance: 10,
            bid,
        },
    )
    .await;
    assert_eq!(
        tx_applied.result,
        Err(RegistryError::FailedFeePayment.into())
    );

    assert_eq!(
        client.free_balance(&poor_actor.public()).await.unwrap(),
        0,
        "Tx author shouldn't have had funds to pay any fees"
    );
}

/// Test that we can transfer money to an org account and that the
/// org owner can transfer money from an org to another account.
#[async_std::test]
async fn org_account_transfer() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob").public();
    let org = create_random_org(&client, &alice).await;

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();

    assert_eq!(client.free_balance(&org.account_id).await.unwrap(), 0);
    let transfer_amount = 2000;
    let transfer_bid = random_balance();
    submit_ok(
        &client,
        &alice,
        message::Transfer {
            recipient: org.account_id,
            balance: transfer_amount,
            bid: transfer_bid,
        },
    )
    .await;
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        transfer_amount
    );
    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - transfer_amount - transfer_bid,
        "Tx author should have paid all fees"
    );

    assert_eq!(client.free_balance(&bob).await.unwrap(), 0);

    let bid = random_balance();
    let tip = bid - BaseFee.value();
    let org_amount = 1000;

    submit_ok(
        &client,
        &alice,
        message::TransferFromOrg {
            org_id: org.id.clone(),
            recipient: bob,
            value: org_amount,
            bid,
        },
    )
    .await;
    assert_eq!(client.free_balance(&bob).await.unwrap(), org_amount);
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_amount - tip
    );
}

#[async_std::test]
/// Test that a transfer from an org account fails if the sender is not an org member.
async fn org_account_transfer_non_member() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let org = create_random_org(&client, &alice).await;

    let bid = random_balance();
    let amount_to_org = 2000;
    submit_ok(
        &client,
        &alice,
        message::Transfer {
            recipient: org.account_id,
            balance: amount_to_org,
            bid,
        },
    )
    .await;
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        amount_to_org
    );

    let bad_actor = key_pair_from_string("BadActor");
    // The bad actor needs funds to issue a tx.
    grant_funds(&client, &alice, bad_actor.public(), 1000).await;
    let bad_actor_balance_before = client.free_balance(&bad_actor.public()).await.unwrap();

    submit_ok(
        &client,
        &bad_actor,
        message::TransferFromOrg {
            org_id: org.id.clone(),
            recipient: bad_actor.public(),
            value: 1000,
            bid: random_balance(),
        },
    )
    .await;

    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        bad_actor_balance_before - BaseFee.value(),
        "The tx author should have paid the base fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        amount_to_org,
        "The org shouldn't have paid any tx fees"
    );
}
