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
async fn create_checkpoint_with_user() {
    let (client, _) = Client::new_emulator();
    let (author, user_id) = key_pair_with_associated_user(&client).await;
    let domain = ProjectDomain::User(user_id);
    create_checkpoint_with_domain(&client, &author, &domain).await;
}

#[async_std::test]
async fn create_checkpoint_with_org() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, _) = register_random_org(&client, &author).await;
    let domain = ProjectDomain::Org(org_id);
    create_checkpoint_with_domain(&client, &author, &domain).await;
}

async fn create_checkpoint_with_domain(
    client: &Client,
    author: &ed25519::Pair,
    domain: &ProjectDomain,
) {
    let (_, project) = create_project(client, author, domain).await;

    let initial_balance = client.free_balance(&author.public()).await.unwrap();
    let project_hash = H256::random();
    let random_fee = random_balance();
    let new_checkpoint_id = submit_ok_with_fee(
        &client,
        &author,
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id: Some(project.current_cp()),
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
        state::Checkpoints1Data::new(Some(project.current_cp()), project_hash),
    );

    assert_eq!(
        client.free_balance(&author.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn set_checkpoint_with_user() {
    let (client, _) = Client::new_emulator();
    let (author, user_id) = key_pair_with_associated_user(&client).await;
    let domain = ProjectDomain::User(user_id);
    set_checkpoint_with_domain(&client, &author, domain, author.public()).await;
}

#[async_std::test]
async fn set_checkpoint_with_org() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, org) = register_random_org(&client, &author).await;
    let domain = ProjectDomain::Org(org_id);
    set_checkpoint_with_domain(&client, &author, domain, org.account_id()).await;
}

async fn set_checkpoint_with_domain(
    client: &Client,
    author: &ed25519::Pair,
    domain: ProjectDomain,
    domain_account: AccountId,
) {
    let (project_name, project) = create_project(client, author, &domain).await;
    let new_checkpoint_id = create_checkpoint(client, author, project.current_cp()).await;
    let initial_balance = client.free_balance(&domain_account).await.unwrap();
    let random_fee = random_balance();

    submit_ok_with_fee(
        client,
        author,
        message::SetCheckpoint {
            project_name: project_name.clone(),
            project_domain: domain.clone(),
            new_checkpoint_id,
        },
        random_fee,
    )
    .await;

    let new_project = client
        .get_project(project_name, domain)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(new_checkpoint_id, new_project.current_cp());
    assert_eq!(
        client.free_balance(&domain_account).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}

#[async_std::test]
async fn set_checkpoint_with_user_without_permission() {
    let (client, _) = Client::new_emulator();
    let (author, user_id) = key_pair_with_associated_user(&client).await;
    let domain = ProjectDomain::User(user_id);
    set_checkpoint_with_domain_without_permission(&client, &author, domain).await;
}

#[async_std::test]
async fn set_checkpoint_with_org_without_permission() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, _) = register_random_org(&client, &author).await;
    let domain = ProjectDomain::Org(org_id);
    set_checkpoint_with_domain_without_permission(&client, &author, domain).await;
}

async fn set_checkpoint_with_domain_without_permission(
    client: &Client,
    author: &ed25519::Pair,
    project_domain: ProjectDomain,
) {
    let (project_name, project) = create_project(client, author, &project_domain).await;
    let new_checkpoint_id = create_checkpoint(client, author, project.current_cp()).await;
    let (bad_actor, _) = key_pair_with_associated_user(&client).await;

    let tx_included = submit_ok(
        &client,
        &bad_actor,
        message::SetCheckpoint {
            project_name: project_name.clone(),
            project_domain: project_domain.clone(),
            new_checkpoint_id,
        },
    )
    .await;

    assert_eq!(
        tx_included.result,
        Err(RegistryError::InsufficientSenderPermissions.into())
    );
    let updated_project = client
        .get_project(project_name, project_domain)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_project.current_cp(), project.current_cp().clone());
    assert_ne!(updated_project.current_cp(), new_checkpoint_id);
}

#[async_std::test]
async fn fail_to_set_nonexistent_checkpoint_with_user() {
    let (client, _) = Client::new_emulator();
    let (author, user_id) = key_pair_with_associated_user(&client).await;
    let domain = ProjectDomain::User(user_id);
    fail_to_set_nonexistent_checkpoint_with_domain(&client, &author, domain).await;
}

#[async_std::test]
async fn fail_to_set_nonexistent_checkpoint_with_org() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, _) = register_random_org(&client, &author).await;
    let domain = ProjectDomain::Org(org_id);
    fail_to_set_nonexistent_checkpoint_with_domain(&client, &author, domain).await;
}

async fn fail_to_set_nonexistent_checkpoint_with_domain(
    client: &Client,
    author: &ed25519::Pair,
    project_domain: ProjectDomain,
) {
    let (project_name, project) = create_project(client, author, &project_domain).await;
    let garbage = CheckpointId::random();

    let tx_included = submit_ok(
        client,
        author,
        message::SetCheckpoint {
            project_name: project_name.clone(),
            project_domain: project_domain.clone(),
            new_checkpoint_id: garbage,
        },
    )
    .await;

    assert_eq!(
        tx_included.result,
        Err(RegistryError::InexistentCheckpointId.into())
    );
    let updated_project = client
        .get_project(project_name, project_domain)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(updated_project.current_cp(), project.current_cp());
    assert_ne!(updated_project.current_cp(), garbage);
}

#[async_std::test]
async fn set_fork_checkpoint_with_user() {
    let (client, _) = Client::new_emulator();
    let (author, user_id) = key_pair_with_associated_user(&client).await;
    let domain = ProjectDomain::User(user_id);
    set_fork_checkpoint_with_domain(&client, &author, domain).await;
}

#[async_std::test]
async fn set_fork_checkpoint_with_org() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, _) = register_random_org(&client, &author).await;
    let domain = ProjectDomain::Org(org_id);
    set_fork_checkpoint_with_domain(&client, &author, domain).await;
}

async fn set_fork_checkpoint_with_domain(
    client: &Client,
    author: &ed25519::Pair,
    project_domain: ProjectDomain,
) {
    let (project_name, project) = create_project(client, author, &project_domain).await;
    let mut current_cp = project.current_cp();
    let mut checkpoints: Vec<CheckpointId> = Vec::new();
    for _ in 0..5 {
        current_cp = create_checkpoint(client, author, current_cp).await;
        checkpoints.push(current_cp);
    }
    let forked_checkpoint_id = create_checkpoint(client, author, checkpoints[2]).await;

    set_checkpoint(
        client,
        author,
        &project_name,
        &project_domain,
        forked_checkpoint_id,
    )
    .await;

    let project_1 = client
        .get_project(project_name, project_domain)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(project_1.current_cp(), forked_checkpoint_id)
}

#[async_std::test]
async fn set_checkpoint_with_user_bad_actor() {
    let (client, _) = Client::new_emulator();
    let (author, user_id) = key_pair_with_associated_user(&client).await;
    let domain = ProjectDomain::User(user_id);
    set_checkpoint_with_domain_bad_actor(&client, &author, domain).await;
}

#[async_std::test]
async fn set_checkpoint_with_org_bad_actor() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, _) = register_random_org(&client, &author).await;
    let domain = ProjectDomain::Org(org_id);
    set_checkpoint_with_domain_bad_actor(&client, &author, domain).await;
}

/// Test that a bad actor can not set a checkpoint of a project he doesn't own.
/// Also test that the bad actor pays the tx fee nonetheless.
async fn set_checkpoint_with_domain_bad_actor(
    client: &Client,
    author: &ed25519::Pair,
    project_domain: ProjectDomain,
) {
    let (bad_actor, _) = key_pair_with_associated_user(&client).await;
    let initial_balance = client.free_balance(&bad_actor.public()).await.unwrap();
    let (project_name, project) = create_project(client, author, &project_domain).await;
    let new_checkpoint_id = create_checkpoint(client, author, project.current_cp()).await;
    let random_fee = random_balance();

    submit_ok_with_fee(
        client,
        &bad_actor,
        message::SetCheckpoint {
            project_name: project_name.clone(),
            project_domain: project_domain.clone(),
            new_checkpoint_id,
        },
        random_fee,
    )
    .await;

    // Check that the project checkpoint was kept untouched.
    let project_after = client
        .get_project(project_name, project_domain)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(project.current_cp(), project_after.current_cp());
    // Check that the bad author paid the fee.
    assert_eq!(
        client.free_balance(&bad_actor.public()).await.unwrap(),
        initial_balance - random_fee,
        "The tx fee was not charged properly."
    );
}
