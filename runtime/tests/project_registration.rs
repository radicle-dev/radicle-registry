/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern project registration.
use radicle_registry_client::*;
use radicle_registry_runtime::fees::{BaseFee, Fee};
use radicle_registry_test_utils::*;

#[async_std::test]
async fn register_project() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let project_hash = H256::random();
    let checkpoint_id = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id: None,
            bid: 10,
        },
    )
    .await
    .result
    .unwrap();

    let register_org = random_register_org_message();
    submit_ok(&client, &alice, register_org.clone()).await;
    let org = client
        .get_org(register_org.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    // The org needs some funds in order to register a project.
    grant_funds(&client, &alice, org.account_id, 1000).await;

    let message = random_register_project_message(org.id.clone(), checkpoint_id);
    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();

    let tx_applied = submit_ok(&client, &alice, message.clone()).await;

    let project = client
        .get_project(message.clone().project_name, message.clone().org_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(project.name.clone(), message.project_name.clone());
    assert_eq!(project.org_id.clone(), message.org_id.clone());
    assert_eq!(project.current_cp.clone(), checkpoint_id);
    assert_eq!(project.metadata.clone(), message.metadata.clone());

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::ProjectRegistered(message.clone().project_name, message.clone().org_id)
            .into()
    );

    let has_project = client
        .list_projects()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == (message.project_name.clone(), message.org_id.clone()));
    assert!(has_project, "Registered project not found in project list");

    let checkpoint_ = state::Checkpoint {
        parent: None,
        hash: project_hash,
    };
    let checkpoint = client.get_checkpoint(checkpoint_id).await.unwrap().unwrap();
    assert_eq!(checkpoint, checkpoint_);

    let org: Org = client
        .get_org(register_org.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(org.projects.len(), 1);
    assert!(
        org.projects.contains(&project.name.clone()),
        "Org does not contain the added project."
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "Tx author should have (only) paid for the base fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before - (message.bid - BaseFee.value()),
        "The org should have (only) paid for the tip",
    );
}

#[async_std::test]
async fn register_project_with_inexistent_org() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let project_hash = H256::random();
    let checkpoint_id = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id: None,
            bid: 10,
        },
    )
    .await
    .result
    .unwrap();

    let inexistent_org_id = random_string32();
    let message = random_register_project_message(inexistent_org_id, checkpoint_id);
    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let tx_applied = submit_ok(&client, &alice, message.clone()).await;

    assert_eq!(tx_applied.result, Err(RegistryError::InexistentOrg.into()));
    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "Tx author should have (only) paid for the base fee"
    );
}

#[async_std::test]
async fn register_project_with_duplicate_id() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let checkpoint_id = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash: H256::random(),
            previous_checkpoint_id: None,
            bid: 10,
        },
    )
    .await
    .result
    .unwrap();

    let org_id = random_string32();
    let register_org = message::RegisterOrg {
        org_id: org_id.clone(),
        bid: 10,
    };
    submit_ok(&client, &alice, register_org.clone()).await;
    let org = client
        .get_org(register_org.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    // The org needs some funds in order to register a project.
    grant_funds(&client, &alice, org.account_id, 1000).await;

    let message = random_register_project_message(org_id.clone(), checkpoint_id);
    submit_ok(&client, &alice, message.clone()).await;

    // Duplicate submission with a different metadata.
    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();

    let registration_2 = submit_ok(
        &client,
        &alice,
        message::RegisterProject {
            metadata: Bytes128::random(),
            ..message.clone()
        },
    )
    .await;

    assert_eq!(
        registration_2.result,
        Err(RegistryError::DuplicateProjectId.into())
    );

    let project = client
        .get_project(message.project_name, message.org_id)
        .await
        .unwrap()
        .unwrap();
    // Assert that the project data was not altered during the
    // attempt to re-register the already existing project.
    assert_eq!(message.metadata, project.metadata);

    let org = client.get_org(org_id).await.unwrap().unwrap();
    // Assert that the number of projects in the involved Org didn't change.
    assert_eq!(org.projects.len(), 1);
    assert!(
        org.projects.contains(&project.name),
        "Registered project not found in the org project list",
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "Tx author should have (only) paid for the base fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before - (message.bid - BaseFee.value()),
        "The org should have (only) paid for the bid",
    );
}

#[async_std::test]
async fn register_project_with_bad_checkpoint() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let checkpoint_id = H256::random();

    let org_id = random_string32();
    let message = random_register_project_message(org_id.clone(), checkpoint_id);
    let register_org = message::RegisterOrg { org_id, bid: 10 };
    submit_ok(&client, &alice, register_org.clone()).await;
    let org = client.get_org(register_org.org_id).await.unwrap().unwrap();
    grant_funds(&client, &alice, org.account_id.clone(), 1000).await;

    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();

    let tx_applied = submit_ok(&client, &alice, message.clone()).await;

    assert_eq!(
        tx_applied.result,
        Err(RegistryError::InexistentCheckpointId.into())
    );

    assert!(client
        .get_project(message.project_name, message.org_id)
        .await
        .unwrap()
        .is_none());

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "Tx author should have (only) paid for the base fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before - (message.bid - BaseFee.value()),
        "The org should have (only) paid for the bid",
    );
}

#[async_std::test]
async fn register_project_with_bad_actor() {
    let client = Client::new_emulator();
    let god_actor = key_pair_from_string("Alice");
    let bad_actor = key_pair_from_string("BadActor");
    // The bad actor needs some funds in order to run transactions.
    grant_funds(&client, &god_actor, bad_actor.public(), 1000).await;

    let org_id = random_string32();
    let register_project = random_register_project_message(org_id.clone(), H256::random());
    let register_org = message::RegisterOrg { org_id, bid: 10 };

    submit_ok(&client, &god_actor, register_org.clone()).await;
    let org = client
        .get_org(register_org.org_id.clone())
        .await
        .unwrap()
        .unwrap();

    let bad_actor_balance_before = client.free_balance(&bad_actor.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();

    let tx_applied = submit_ok(&client, &bad_actor, register_project.clone()).await;

    assert_eq!(
        tx_applied.result,
        Err(RegistryError::InsufficientSenderPermissions.into())
    );

    assert!(client
        .get_project(register_project.project_name, register_project.org_id)
        .await
        .unwrap()
        .is_none());

    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        bad_actor_balance_before - BaseFee.value(),
        "Tx author should have (only) paid for the base fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before,
        "The org shouldn't have paid for any fees",
    );
}

#[async_std::test]
async fn register_project_with_insufficient_funds_author() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let poor_actor = key_pair_from_string("Poor");

    let org_id = random_string32();
    let register_org = message::RegisterOrg {
        org_id: org_id.clone(),
        bid: 10,
    };
    submit_ok(&client, &alice, register_org.clone()).await;
    let org = client.get_org(org_id.clone()).await.unwrap().unwrap();

    let register_project = random_register_project_message(org_id.clone(), H256::random());

    let poor_actor_balance_before = client.free_balance(&poor_actor.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();

    let tx_applied = submit_ok(&client, &poor_actor, register_project.clone()).await;

    assert_eq!(
        tx_applied.result,
        Err(RegistryError::FailedFeePayment.into())
    );

    assert_eq!(
        client.free_balance(&poor_actor.public()).await.unwrap(),
        poor_actor_balance_before,
        "Tx author should have had no funds to pay for any fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before,
        "The org shouldn't have paid for any fees",
    );
}

#[async_std::test]
async fn register_project_with_insufficient_funds_org() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");

    let project_hash = H256::random();
    let checkpoint_id = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id: None,
            bid: 10,
        },
    )
    .await
    .result
    .unwrap();

    let register_org = random_register_org_message();
    submit_ok(&client, &alice, register_org.clone()).await;
    let org = client
        .get_org(register_org.org_id.clone())
        .await
        .unwrap()
        .unwrap();

    let message = random_register_project_message(org.id.clone(), checkpoint_id);
    let alice_balance_before = client.free_balance(&alice.public()).await.unwrap();
    let org_balance_before = client.free_balance(&org.account_id).await.unwrap();
    assert_eq!(org_balance_before, 0);

    let tx_applied = submit_ok(&client, &alice, message.clone()).await;
    assert_eq!(
        tx_applied.result,
        Err(RegistryError::FailedFeePayment.into())
    );

    assert_eq!(
        client.free_balance(&alice.public()).await.unwrap(),
        alice_balance_before - BaseFee.value(),
        "Tx author should have (only) paid for the base fee"
    );
    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        org_balance_before,
        "The org should have had no funds to pay the tip",
    );
}
