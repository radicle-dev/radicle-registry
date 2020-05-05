// Radicle Registry
// Copyright (C) 2019 Monadic GmbH <radicle@monadic.xyz>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as
// published by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
        message::Transfer {
            recipient: bob,
            balance: balance_alice + 1,
        },
    )
    .await;
    assert!(tx_applied.result.is_err());
}

// Test that we can transfer any amount within a reasonable range.
// Affected by the [crate::ExistentialDeposit] parameter.
#[async_std::test]
async fn transfer_any_amount() {
    let client = Client::new_emulator();
    let donator = key_pair_from_string("Alice");
    let receipient = key_pair_from_string("Bob").public();

    for amount in (1..10000).step_by(500) {
        let tx_applied = submit_ok(
            &client,
            &donator,
            message::Transfer {
                recipient: receipient,
                balance: amount,
            },
        )
        .await;
        assert_eq!(
            tx_applied.result,
            Ok(()),
            "Failed to transfer {} μRAD",
            amount
        );
    }
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
    let alice_initial_balance = client.free_balance(&alice.public()).await.unwrap();
    let random_fee = random_balance();
    let transfer_amount = 2000;
    submit_ok_with_fee(
        &client,
        &alice,
        message::Transfer {
            recipient: org.account_id,
            balance: transfer_amount,
        },
        random_fee,
    )
    .await;
    assert_eq!(client.free_balance(&org.account_id).await.unwrap(), 2000);
    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_initial_balance - transfer_amount - random_fee,
        "The tx fee was not charged properly."
    );

    assert_eq!(client.free_balance(&bob).await.unwrap(), 0);
    let initial_balance_org = client.free_balance(&org.account_id).await.unwrap();
    let org_transfer_fee = random_balance();
    let org_transfer_amount = 1000;
    submit_ok_with_fee(
        &client,
        &alice,
        message::TransferFromOrg {
            org_id: org.id.clone(),
            recipient: bob,
            value: org_transfer_amount,
        },
        org_transfer_fee,
    )
    .await;
    assert_eq!(client.free_balance(&bob).await.unwrap(), 1000);
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        initial_balance_org - org_transfer_amount - org_transfer_fee
    );
}

#[async_std::test]
/// Test that a transfer from an org account fails if the sender is not an org member.
async fn org_account_transfer_non_member() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let org = create_random_org(&client, &alice).await;

    submit_ok(
        &client,
        &alice,
        message::Transfer {
            recipient: org.account_id,
            balance: 2000,
        },
    )
    .await;
    assert_eq!(client.free_balance(&org.account_id).await.unwrap(), 2000);

    let bad_actor = key_pair_from_string("BadActor");
    let initial_balance = 1000;
    // The bad actor needs funds to submit transactions.
    transfer(&client, &alice, bad_actor.public(), initial_balance).await;

    let random_fee = random_balance();
    submit_ok_with_fee(
        &client,
        &bad_actor,
        message::TransferFromOrg {
            org_id: org.id.clone(),
            recipient: bad_actor.public(),
            value: 1000,
        },
        random_fee,
    )
    .await;

    assert_eq!(client.free_balance(&org.account_id).await.unwrap(), 2000);
    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}
