/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern orgs registration.
use radicle_registry_client::*;
use radicle_registry_runtime::fees::{BaseFee, Fee};
use radicle_registry_test_utils::*;

#[async_std::test]
async fn register_org() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_org_message = random_register_org_message();
    let initial_balance = client.free_balance(&alice.public()).await.unwrap();

    let tx_applied = submit_ok(&client, &alice, register_org_message.clone()).await;

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::OrgRegistered(register_org_message.org_id.clone()).into()
    );
    assert_eq!(tx_applied.result, Ok(()));

    assert!(
        org_exists(&client, register_org_message.org_id.clone()).await,
        "Org not found in orgs list"
    );

    let org: Org = client
        .get_org(register_org_message.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(org.id, register_org_message.org_id);
    assert_eq!(org.members, vec![alice.public()]);
    assert!(org.projects.is_empty());

    let balance_after_registration = client.free_balance(&alice.public()).await.unwrap();
    assert_eq!(
        balance_after_registration,
        initial_balance - register_org_message.bid,
        "The tx fees are not being correctly charged from the tx author"
    );
}

#[async_std::test]
async fn register_with_duplicated_org_id() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_org_message = random_register_org_message();
    let initial_balance = client.free_balance(&alice.public()).await.unwrap();

    let tx_applied_once = submit_ok(&client, &alice, register_org_message.clone()).await;
    assert_eq!(tx_applied_once.result, Ok(()));

    let tx_applied_twice = submit_ok(&client, &alice, register_org_message.clone()).await;
    assert_eq!(
        tx_applied_twice.result,
        Err(RegistryError::DuplicateOrgId.into())
    );

    let balance_after_registration = client.free_balance(&alice.public()).await.unwrap();
    assert_eq!(
        balance_after_registration,
        initial_balance - 2 * register_org_message.bid,
        "The tx fees are not being correctly charged from the tx author"
    );
}

#[async_std::test]
/// Test that if the tx author does not have enough funds to pay
/// the tx fees, the org won't be registered.
async fn register_with_insufficient_funds() {
    let client = Client::new_emulator();
    let poor_actor = key_pair_from_string("Poor");
    let register_org_message = random_register_org_message();
    assert_eq!(client.free_balance(&poor_actor.public()).await.unwrap(), 0);

    let tx_applied_once = submit_ok(&client, &poor_actor, register_org_message.clone()).await;
    assert_eq!(
        tx_applied_once.result,
        Err(RegistryError::FailedFeePayment.into())
    );
}

#[async_std::test]
/// Test that if the tx author does not have enough funds to pay
/// the tx fees, the org won't be registered.
async fn register_with_insufficient_bid() {
    let client = Client::new_emulator();
    let cheap_actor = key_pair_from_string("Cheap");
    assert_eq!(client.free_balance(&cheap_actor.public()).await.unwrap(), 0);

    let register_org_message = message::RegisterOrg {
        org_id: random_string32(),
        bid: 0, // insufficient to cover mandatory costs
    };

    let tx_applied_once = submit_ok(&client, &cheap_actor, register_org_message.clone()).await;
    assert_eq!(
        tx_applied_once.result,
        Err(RegistryError::InsufficientBid.into())
    );
}

#[async_std::test]
async fn unregister_org() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_org_message = random_register_org_message();

    let tx_applied = submit_ok(&client, &alice, register_org_message.clone()).await;

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::OrgRegistered(register_org_message.org_id.clone()).into()
    );
    assert_eq!(tx_applied.result, Ok(()));

    assert!(
        org_exists(&client, register_org_message.org_id.clone()).await,
        "Org not found in orgs list"
    );

    // Unregister
    let org = client
        .get_org(register_org_message.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    grant_funds(&client, &alice, org.account_id, 1000).await;

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();
    let bid = random_balance();

    let unregister_org_message = message::UnregisterOrg {
        org_id: org.id.clone(),
        bid,
    };
    let tx_unregister_applied = submit_ok(&client, &alice, unregister_org_message.clone()).await;
    assert_eq!(tx_unregister_applied.result, Ok(()));
    assert!(
        !org_exists(&client, org.id.clone()).await,
        "The org was not expected to exist"
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "Alice should have (only) paid for the base fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before - (bid - BaseFee.value()),
        "Org should have (only) paid for the tip"
    );
}

#[async_std::test]
async fn unregister_org_bad_sender() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let bad_actor = key_pair_from_string("BadActor");
    grant_funds(&client, &alice, bad_actor.public(), 1000).await;

    let register_org_message = random_register_org_message();

    let tx_applied = submit_ok(&client, &alice, register_org_message.clone()).await;

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::OrgRegistered(register_org_message.org_id.clone()).into()
    );
    assert_eq!(tx_applied.result, Ok(()));

    assert!(
        org_exists(&client, register_org_message.org_id.clone()).await,
        "Org not found in orgs list"
    );

    let org = client
        .get_org(register_org_message.org_id.clone())
        .await
        .unwrap()
        .unwrap();

    // Unregister

    let bad_actor_balance_before = client.free_balance(&bad_actor.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();

    let bid = random_balance();
    let unregister_org_message = message::UnregisterOrg {
        org_id: org.id.clone(),
        bid,
    };

    let tx_unregister_applied =
        submit_ok(&client, &bad_actor, unregister_org_message.clone()).await;
    assert_eq!(
        tx_unregister_applied.result,
        Err(RegistryError::UnregisterableOrg.into())
    );
    assert!(
        org_exists(&client, register_org_message.org_id.clone()).await,
        "Org not found in orgs list"
    );

    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        bad_actor_balance_before - BaseFee.value(),
        "The bad actor should have paid for the base fee."
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before,
        "The org shouldn't have paid for a mal intentioned tx involving it."
    );
}

#[async_std::test]
async fn unregister_inexistent_org() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();

    let random_inexistent_org_id = random_string32();
    let bid = random_balance();
    let unregister_org_message = message::UnregisterOrg {
        org_id: random_inexistent_org_id.clone(),
        bid,
    };

    let tx_unregister_applied = submit_ok(&client, &alice, unregister_org_message.clone()).await;
    assert_eq!(
        tx_unregister_applied.result,
        Err(RegistryError::InexistentOrg.into())
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "The tx author should have paid for the base fee."
    );
}

#[async_std::test]
async fn unregister_org_with_projects() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let org_id = random_string32();
    let random_project = create_project_with_checkpoint(org_id.clone(), &client, &alice).await;

    assert!(
        org_exists(&client, random_project.org_id.clone()).await,
        "Org not found in orgs list"
    );

    let org = client
        .get_org(random_project.org_id.clone())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(org.projects.len(), 1);

    // Unregister
    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();

    let bid = random_balance();
    let unregister_org_message = message::UnregisterOrg {
        org_id: random_project.org_id.clone(),
        bid,
    };
    let tx_unregister_applied = submit_ok(&client, &alice, unregister_org_message.clone()).await;

    assert_eq!(
        tx_unregister_applied.result,
        Err(RegistryError::UnregisterableOrg.into())
    );
    assert!(
        org_exists(&client, random_project.org_id.clone()).await,
        "Org not found in orgs list"
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "The tx author should have paid for the base fee."
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before,
        "The org shouldn't have paid for any fees"
    );
}

/// Test that if the tx author does not have enough funds to pay
/// the tx fees, the tx won't run.
#[async_std::test]
async fn unregister_with_insufficient_funds_author() {
    let client = Client::new_emulator();
    let poor_actor = key_pair_from_string("Poor");
    assert_eq!(client.free_balance(&poor_actor.public()).await.unwrap(), 0);

    let random_inexistent_org_id = random_string32();
    let bid = random_balance();
    let unregister_org_message = message::UnregisterOrg {
        org_id: random_inexistent_org_id.clone(),
        bid,
    };

    let tx_unregister_applied =
        submit_ok(&client, &poor_actor, unregister_org_message.clone()).await;
    assert_eq!(
        tx_unregister_applied.result,
        Err(RegistryError::FailedFeePayment.into())
    );

    assert_eq!(client.free_balance(&poor_actor.public()).await.unwrap(), 0);
}

#[async_std::test]
async fn unregister_with_insufficient_funds_org() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_org_message = random_register_org_message();

    let tx_applied = submit_ok(&client, &alice, register_org_message.clone()).await;
    assert_eq!(tx_applied.result, Ok(()));

    // Unregister
    let org = client
        .get_org(register_org_message.org_id.clone())
        .await
        .unwrap()
        .unwrap();

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();
    assert_eq!(org_balance_before, 0);

    let bid = random_balance();
    let unregister_org_message = message::UnregisterOrg {
        org_id: org.id.clone(),
        bid,
    };
    let tx_unregister_applied = submit_ok(&client, &alice, unregister_org_message.clone()).await;
    assert_eq!(
        tx_unregister_applied.result,
        Err(RegistryError::FailedFeePayment.into())
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "Tx author should have (only) paid for the base fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        0,
        "The org balance should have remained 0.",
    );
}

async fn org_exists(client: &Client, org_id: OrgId) -> bool {
    client
        .list_orgs()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == org_id.clone())
}
