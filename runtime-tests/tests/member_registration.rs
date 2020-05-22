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
/// The tests in this module concern project registration.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn register_member() {
    let (client, _) = Client::new_emulator();
    let (author, author_id) = key_pair_with_associated_user(&client).await;
    let (_, member_user_id) = key_pair_with_associated_user(&client).await;

    let register_org = random_register_org_message();
    submit_ok(&client, &author, register_org.clone()).await;

    // The org needs funds to submit transactions.
    let org = client
        .get_org(register_org.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    let initial_balance = 1000;
    transfer(&client, &author, org.account_id, initial_balance).await;

    let random_fee = random_balance();
    let message = message::RegisterMember {
        org_id: register_org.org_id.clone(),
        user_id: member_user_id,
    };
    let tx_applied = submit_ok_with_fee(&client, &author, message.clone(), random_fee).await;

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::MemberRegistered(message.clone().user_id, message.clone().org_id).into()
    );

    // Fetch the org again
    let re_org = client
        .get_org(message.clone().org_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(re_org.members.len(), 2);
    assert!(
        re_org.members.contains(&author_id),
        "Org does not contain the founding member."
    );
    assert!(
        re_org.members.contains(&message.clone().user_id),
        "Org does not contain the added member."
    );

    assert_eq!(
        client.free_balance(&re_org.account_id).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn register_member_with_inexistent_org() {
    let (client, _) = Client::new_emulator();
    let (author, user_id) = key_pair_with_associated_user(&client).await;
    let initial_balance = client.free_balance(&author.public()).await.unwrap();

    let random_fee = random_balance();
    let inexistent_org_id = random_id();
    let message = message::RegisterMember {
        org_id: inexistent_org_id,
        user_id,
    };
    let tx_applied = submit_ok_with_fee(&client, &author, message.clone(), random_fee).await;

    assert_eq!(tx_applied.result, Err(RegistryError::InexistentOrg.into()));

    // Check that the author payed for the transaction anyway.
    assert_eq!(
        client.free_balance(&author.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn register_member_with_bad_actor() {
    let (client, _) = Client::new_emulator();
    let (good_actor, good_actor_id) = key_pair_with_associated_user(&client).await;
    let (bad_actor, bad_actor_id) = key_pair_with_associated_user(&client).await;

    // The good actor creates an org, of which becomes its single member.
    let org_id = random_id();
    let register_org = message::RegisterOrg {
        org_id: org_id.clone(),
    };
    submit_ok(&client, &good_actor, register_org.clone()).await;

    // The bad actor attempts to register themselves as a member within that org.
    let initial_balance = client.free_balance(&bad_actor.public()).await.unwrap();
    let register_member = message::RegisterMember {
        org_id,
        user_id: bad_actor_id,
    };
    let random_fee = random_balance();
    let tx_applied =
        submit_ok_with_fee(&client, &bad_actor, register_member.clone(), random_fee).await;

    assert_eq!(
        tx_applied.result,
        Err(RegistryError::InsufficientSenderPermissions.into())
    );

    // Check that the bad actor payed for the transaction anyway.
    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );

    // Re-fetch the org and check that the bad actor was not added as a member
    let re_org = client
        .get_org(register_member.clone().org_id)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(re_org.members, vec![good_actor_id]);
}

#[async_std::test]
async fn register_duplicate_member() {
    let (client, _) = Client::new_emulator();
    let (author, author_id) = key_pair_with_associated_user(&client).await;

    // Register the org.
    let org_id = random_id();
    let register_org = message::RegisterOrg {
        org_id: org_id.clone(),
    };
    submit_ok(&client, &author, register_org.clone()).await;

    // The org needs funds to submit transactions.
    let org = client
        .get_org(register_org.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    let initial_balance = 1000;
    transfer(&client, &author, org.account_id, initial_balance).await;

    // The author attempts to register themselves as a member within that org again.
    let register_member = message::RegisterMember {
        org_id,
        user_id: author_id.clone(),
    };
    let random_fee = random_balance();
    let tx_applied =
        submit_ok_with_fee(&client, &author, register_member.clone(), random_fee).await;

    assert_eq!(tx_applied.result, Err(RegistryError::AlreadyAMember.into()));

    // Re-fetch the org
    let re_org = client
        .get_org(register_member.clone().org_id)
        .await
        .unwrap()
        .unwrap();

    // Check that the org payed for the transaction anyway.
    assert_eq!(
        client.free_balance(&re_org.account_id).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );

    // Check that the author was not added again
    assert_eq!(re_org.members, vec![author_id]);
}

#[async_std::test]
async fn register_nonexistent_user() {
    let (client, _) = Client::new_emulator();
    let (author, author_id) = key_pair_with_associated_user(&client).await;

    // Register the org.
    let org_id = random_id();
    let register_org = message::RegisterOrg {
        org_id: org_id.clone(),
    };
    submit_ok(&client, &author, register_org.clone()).await;

    // The org needs funds to submit transactions.
    let org = client
        .get_org(register_org.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    let initial_balance = 1000;
    transfer(&client, &author, org.account_id, initial_balance).await;

    // Attempt to register a non-existent user as a member.
    let register_member = message::RegisterMember {
        org_id,
        user_id: random_id(),
    };
    let random_fee = random_balance();
    let tx_applied =
        submit_ok_with_fee(&client, &author, register_member.clone(), random_fee).await;

    assert_eq!(tx_applied.result, Err(RegistryError::InexistentUser.into()));

    // Re-fetch the org
    let re_org = client
        .get_org(register_member.clone().org_id)
        .await
        .unwrap()
        .unwrap();

    // Check that the org payed for the transaction anyway.
    assert_eq!(
        client.free_balance(&re_org.account_id).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );

    // Check that no new member was added
    assert_eq!(re_org.members, vec![author_id]);
}
