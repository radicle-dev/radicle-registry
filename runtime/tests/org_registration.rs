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
        RegistryEvent::OrgRegistered(register_org_message.id.clone()).into()
    );
    assert_eq!(tx_applied.result, Ok(()));

    let has_org = client
        .list_orgs()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == register_org_message.id);
    assert!(has_org, "Registered org not found in orgs list");

    let org: Org = client
        .get_org(register_org_message.id.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(org.id, register_org_message.id);
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
        RegistryEvent::OrgRegistered(register_org_message.id.clone()).into()
    );
    assert_eq!(tx_applied.result, Ok(()));

    let has_org = client
        .list_orgs()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == register_org_message.id.clone());
    assert!(has_org, "Registered org not found in orgs list");

    // Unregister
    let unregister_org_message = message::UnregisterOrg {
        id: register_org_message.id.clone(),
    };
    let tx_unregister_applied = submit_ok(&client, &alice, unregister_org_message.clone()).await;
    assert_eq!(tx_unregister_applied.result, Ok(()));

    let org_is_gone = !client
        .list_orgs()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == register_org_message.id.clone());
    assert!(org_is_gone, "Registered org not found in orgs list");
}

#[async_std::test]
async fn unregister_org_bad_sender() {
    let client = Client::new_emulator();
    let alice = key_pair_from_string("Alice");
    let register_org_message = random_register_org_message();

    let tx_applied = submit_ok(&client, &alice, register_org_message.clone()).await;

    assert_eq!(
        tx_applied.events[0],
        RegistryEvent::OrgRegistered(register_org_message.id.clone()).into()
    );
    assert_eq!(tx_applied.result, Ok(()));

    let has_org = client
        .list_orgs()
        .await
        .unwrap()
        .iter()
        .any(|id| *id == register_org_message.id.clone());
    assert!(has_org, "Registered org not found in orgs list");

    // Unregister
    let unregister_org_message = message::UnregisterOrg {
        id: register_org_message.id.clone(),
    };
    let bad_actor = key_pair_from_string("BadActor");
    let tx_unregister_applied =
        submit_ok(&client, &bad_actor, unregister_org_message.clone()).await;
    assert_eq!(
        tx_unregister_applied.result,
        Err(RegistryError::UnregisterableOrg.into())
    );
}

// TODO(nuno): Test unregister_org_with_projects once possible.
