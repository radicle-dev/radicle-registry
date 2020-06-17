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
/// The tests in this module concern checkpoint creation and setting project
/// checkpoints.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn create_checkpoint() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let org_id = random_id();
    let project =
        create_project_with_checkpoint(&ProjectRegistrant::Org(org_id.clone()), &client, &author)
            .await;

    let initial_balance = client.free_balance(&author.public()).await.unwrap();
    let project_hash = H256::random();
    let random_fee = random_balance();
    let new_checkpoint_id = submit_ok_with_fee(
        &client,
        &author,
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id: Some(project.current_cp),
        },
        random_fee,
    )
    .await
    .result
    .unwrap();

    let checkpoint = client
        .get_checkpoint(new_checkpoint_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        checkpoint,
        Checkpoint::new(state::Checkpoints1Data::new(
            Some(project.current_cp),
            project_hash
        )),
    );

    assert_eq!(
        client.free_balance(&author.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn set_checkpoint() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let org_id = random_id();
    let project_registrant = ProjectRegistrant::Org(org_id.clone());
    let project =
        create_project_with_checkpoint(&ProjectRegistrant::Org(org_id.clone()), &client, &author)
            .await;
    let project_name = project.clone().name;

    let project_hash2 = H256::random();
    let new_checkpoint_id = submit_ok(
        &client,
        &author,
        message::CreateCheckpoint {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(project.current_cp),
        },
    )
    .await
    .result
    .unwrap();

    let org = client.get_org(org_id.clone()).await.unwrap().unwrap();
    let initial_balance = client.free_balance(&org.account_id).await.unwrap();
    let random_fee = random_balance();
    submit_ok_with_fee(
        &client,
        &author,
        message::SetCheckpoint {
            project_name: project.name,
            project_registrant: project_registrant.clone(),
            new_checkpoint_id,
        },
        random_fee,
    )
    .await;

    let new_project = client
        .get_project(project_name, project_registrant)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(new_checkpoint_id, new_project.current_cp);

    assert_eq!(
        client.free_balance(&org.account_id).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn set_checkpoint_without_permission() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let org_id = random_id();
    let project_registrant = ProjectRegistrant::Org(org_id.clone());
    let project = create_project_with_checkpoint(&project_registrant, &client, &author).await;
    let project_name = project.name.clone();

    let project_hash2 = H256::random();
    let new_checkpoint_id = submit_ok(
        &client,
        &author,
        message::CreateCheckpoint {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(project.current_cp),
        },
    )
    .await
    .result
    .unwrap();

    let (bad_actor, _) = key_pair_with_associated_user(&client).await;

    let tx_included = submit_ok(
        &client,
        &bad_actor,
        message::SetCheckpoint {
            project_name: project.name,
            project_registrant: project.registrant,
            new_checkpoint_id,
        },
    )
    .await;

    let updated_project = client
        .get_project(project_name, project_registrant.clone())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        tx_included.result,
        Err(RegistryError::InsufficientSenderPermissions.into())
    );
    assert_eq!(updated_project.current_cp, project.current_cp.clone());
    assert_ne!(updated_project.current_cp, new_checkpoint_id);
}

#[async_std::test]
async fn fail_to_set_nonexistent_checkpoint() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let org_id = random_id();
    let project_registrant = ProjectRegistrant::Org(org_id.clone());
    let project = create_project_with_checkpoint(&project_registrant, &client, &author).await;
    let project_name = project.name.clone();
    let garbage = CheckpointId::random();

    let tx_included = submit_ok(
        &client,
        &author,
        message::SetCheckpoint {
            project_name: project.name,
            project_registrant: project.registrant,
            new_checkpoint_id: garbage,
        },
    )
    .await;

    assert_eq!(
        tx_included.result,
        Err(RegistryError::InexistentCheckpointId.into())
    );
    let updated_project = client
        .get_project(project_name, project_registrant)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_project.current_cp, project.current_cp);
    assert_ne!(updated_project.current_cp, garbage);
}

#[async_std::test]
async fn set_fork_checkpoint() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let org_id = random_id();
    let project_registrant = ProjectRegistrant::Org(org_id.clone());
    let project = create_project_with_checkpoint(&project_registrant, &client, &author).await;

    let project_name = project.name.clone();
    let mut current_cp = project.current_cp;

    // How many checkpoints to create.
    let n = 5;
    let mut checkpoints: Vec<CheckpointId> = Vec::with_capacity(n);
    for _ in 0..n {
        let new_checkpoint_id = submit_ok(
            &client,
            &author,
            message::CreateCheckpoint {
                project_hash: H256::random(),
                previous_checkpoint_id: (Some(current_cp)),
            },
        )
        .await
        .result
        .unwrap();
        current_cp = new_checkpoint_id;
        checkpoints.push(new_checkpoint_id);
    }

    let forked_checkpoint_id = submit_ok(
        &client,
        &author,
        message::CreateCheckpoint {
            project_hash: H256::random(),
            previous_checkpoint_id: (Some(checkpoints[2])),
        },
    )
    .await
    .result
    .unwrap();

    submit_ok(
        &client,
        &author,
        message::SetCheckpoint {
            project_name: project.name,
            project_registrant: project.registrant,
            new_checkpoint_id: forked_checkpoint_id,
        },
    )
    .await;

    let project_1 = client
        .get_project(project_name, project_registrant)
        .await
        .unwrap()
        .unwrap();

    assert_eq!(project_1.current_cp, forked_checkpoint_id)
}

// Test that a bad actor can not set a checkpoint of
// a project from an org they do not belong to. Also,
// test that the bad actor pays the tx fee nonetheless.
#[async_std::test]
async fn set_checkpoint_bad_actor() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    let org_id = random_id();
    let project_registrant = ProjectRegistrant::Org(org_id.clone());
    let project = create_project_with_checkpoint(&project_registrant, &client, &author).await;
    let project_name = project.clone().name;

    let project_hash2 = H256::random();
    let new_checkpoint_id = submit_ok(
        &client,
        &author,
        message::CreateCheckpoint {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(project.current_cp),
        },
    )
    .await
    .result
    .unwrap();

    let bad_actor = key_pair_from_string("BadActor");
    let initial_balance = 1000;
    // The bad actor needs funds to submit transactions.
    transfer(&client, &author, bad_actor.public(), initial_balance).await;

    let random_fee = random_balance();
    submit_ok_with_fee(
        &client,
        &bad_actor,
        message::SetCheckpoint {
            project_name: project.name,
            project_registrant: project.registrant,
            new_checkpoint_id,
        },
        random_fee,
    )
    .await;

    // Check that the project checkpoint was kept untouched.
    let project_after = client
        .get_project(project_name, project_registrant)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(project.current_cp, project_after.current_cp);

    // Check that the bad author paid the fee.
    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}
