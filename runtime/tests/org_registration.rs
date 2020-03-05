/// Runtime tests implemented with [MemoryClient].
///
/// High-level runtime tests that only use [MemoryClient] and treat the runtime as a black box.
///
/// The tests in this module concern orgs registration.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn register_org() {
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

    let org: Org = client
        .get_org(register_org_message.org_id.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(org.id, register_org_message.org_id);
    assert_eq!(org.members, vec![alice.public()]);
    assert!(org.projects.is_empty());
}

#[async_std::test]
async fn register_with_duplicated_org_id() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_org_message = random_register_org_message();

    let tx_applied_once = submit_ok(&client, &alice, register_org_message.clone()).await;
    assert_eq!(tx_applied_once.result, Ok(()));

    let tx_applied_twice = submit_ok(&client, &alice, register_org_message.clone()).await;
    assert_eq!(
        tx_applied_twice.result,
        Err(RegistryError::DuplicateOrgId.into())
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

    let unregister_org_message = message::UnregisterOrg {
        org_id: org.id.clone(),
        bid: 100,
    };
    let tx_unregister_applied = submit_ok(&client, &alice, unregister_org_message.clone()).await;
    assert_eq!(tx_unregister_applied.result, Ok(()));

    assert!(
        !org_exists(&client, org.id.clone()).await,
        "The org was not expected to exist"
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

    // Unregister
    let unregister_org_message = message::UnregisterOrg {
        org_id: register_org_message.org_id.clone(),
        bid: 100,
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
    let unregister_org_message = message::UnregisterOrg {
        org_id: random_project.org_id.clone(),
        bid: 100,
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
}

async fn org_exists(client: &Client, org_id: OrgId) -> bool {
    client
        .list_orgs()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == org_id.clone())
}
