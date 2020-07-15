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
/// The tests in this module concern orgs registration.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn test_available() {
    let (client, _) = Client::new_emulator();
    let status = client.get_id_status(&random_id()).await.unwrap();

    assert_eq!(status, IdStatus::Available);
}

/// Test that an Id is Taken by an org
#[async_std::test]
async fn test_taken_by_org() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;
    let (org_id, _) = register_random_org(&client, &author).await;

    let status = client.get_id_status(&org_id).await.unwrap();
    assert_eq!(status, IdStatus::Taken);
}

/// Test that an Id is Taken by a user
#[async_std::test]
async fn test_taken_by_user() {
    let (client, _) = Client::new_emulator();
    let (_, user_id) = key_pair_with_associated_user(&client).await;

    let status = client.get_id_status(&user_id).await.unwrap();
    assert_eq!(status, IdStatus::Taken);
}

/// Test that an Id is Retired once unregistered by an org
#[async_std::test]
async fn test_retired_by_org() {
    let (client, _) = Client::new_emulator();
    let (author, _) = key_pair_with_associated_user(&client).await;

    // Register org
    let (org_id, _) = register_random_org(&client, &author).await;

    // Unregister org
    let unregister_org_message = message::UnregisterOrg {
        org_id: org_id.clone(),
    };
    let tx_unregister_applied = submit_ok(&client, &author, unregister_org_message.clone()).await;
    assert_eq!(tx_unregister_applied.result, Ok(()));

    let status = client.get_id_status(&org_id).await.unwrap();
    assert_eq!(status, IdStatus::Retired);
}

/// Test that an Id is Retired once unregistered by a user
#[async_std::test]
async fn test_retired_by_user() {
    let (client, _) = Client::new_emulator();
    // Register user
    let (author, user_id) = key_pair_with_associated_user(&client).await;

    // Unregister user
    let unregister_user_message = message::UnregisterUser {
        user_id: user_id.clone(),
    };
    let tx_unregister_applied = submit_ok(&client, &author, unregister_user_message.clone()).await;
    assert_eq!(tx_unregister_applied.result, Ok(()));

    let status = client.get_id_status(&user_id).await.unwrap();
    assert_eq!(status, IdStatus::Retired);
}
