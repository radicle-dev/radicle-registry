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
    let (client, _) = Client::new_emulator();
    let alice = key_pair_with_funds(&client).await;
    let bob = key_pair_with_funds(&client).await.public();

    let balance_alice = client.free_balance(&alice.public()).await.unwrap();
    let tx_included = submit_ok(
        &client,
        &alice,
        message::Transfer {
            recipient: bob,
            amount: balance_alice + 1,
        },
    )
    .await;
    assert!(tx_included.result.is_err());
}

// Test that we can transfer any amount within a reasonable range.
// Affected by the [crate::ExistentialDeposit] parameter.
#[async_std::test]
async fn transfer_any_amount() {
    let (client, _) = Client::new_emulator();
    let donator = key_pair_with_funds(&client).await;
    let receipient = ed25519::Pair::generate().0.public();

    for amount in (1..10000).step_by(500) {
        let tx_included = submit_ok(
            &client,
            &donator,
            message::Transfer {
                recipient: receipient,
                amount,
            },
        )
        .await;
        assert_eq!(
            tx_included.result,
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
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let bob = ed25519::Pair::generate().0.public();
    let (org_id, org) = register_random_org(&client, &author).await;

    let org_inigial_balance = client.free_balance(&org.account_id()).await.unwrap();
    let alice_initial_balance = client.free_balance(&author.public()).await.unwrap();
    let random_fee = random_balance();
    let transfer_amount = 2000;
    submit_ok_with_fee(
        &client,
        &author,
        message::Transfer {
            recipient: org.account_id(),
            amount: transfer_amount,
        },
        random_fee,
    )
    .await;
    let org_balance_after_transfer = client.free_balance(&org.account_id()).await.unwrap();
    assert_eq!(
        org_balance_after_transfer,
        org_inigial_balance + transfer_amount
    );
    assert_eq!(
        client.free_balance(&author.public()).await.unwrap(),
        alice_initial_balance - transfer_amount - random_fee,
        "The tx fee was not charged properly."
    );

    assert_eq!(client.free_balance(&bob).await.unwrap(), 0);
    let initial_balance_org = client.free_balance(&org.account_id()).await.unwrap();
    let org_transfer_fee = random_balance();
    let org_transfer_amount = 1000;
    submit_ok_with_fee(
        &client,
        &author,
        message::TransferFromOrg {
            org_id,
            recipient: bob,
            amount: org_transfer_amount,
        },
        org_transfer_fee,
    )
    .await;
    assert_eq!(client.free_balance(&bob).await.unwrap(), 1000);
    assert_eq!(
        client.free_balance(&org.account_id()).await.unwrap(),
        initial_balance_org - org_transfer_amount - org_transfer_fee
    );
}

#[async_std::test]
/// Test that a transfer from an org account fails if the sender is not an org member.
async fn org_account_transfer_non_member() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, org) = register_random_org(&client, &author).await;

    let initial_balance = client.free_balance(&org.account_id()).await.unwrap();

    let bad_actor = ed25519::Pair::generate().0;
    // The bad actor needs funds to submit transactions.
    transfer(&client, &author, bad_actor.public(), 1000).await;

    let random_fee = random_balance();
    submit_ok_with_fee(
        &client,
        &bad_actor,
        message::TransferFromOrg {
            org_id,
            recipient: bad_actor.public(),
            amount: 1000,
        },
        random_fee,
    )
    .await;

    assert_eq!(
        client.free_balance(&org.account_id()).await.unwrap(),
        initial_balance,
    );
    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}
