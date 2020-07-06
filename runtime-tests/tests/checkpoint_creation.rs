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
/// The tests in this module concern checkpoint creation.
use radicle_registry_client::*;
use radicle_registry_test_utils::*;

#[async_std::test]
async fn create_checkpoint() {
    let (client, _) = Client::new_emulator();
    let alice = key_pair_with_funds(&client).await;

    let project_hash1 = H256::random();
    let checkpoint_id1 = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash: project_hash1,
            previous_checkpoint_id: None,
        },
    )
    .await
    .result
    .unwrap();

    let project_hash2 = H256::random();
    let checkpoint_id2 = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash: project_hash2,
            previous_checkpoint_id: Some(checkpoint_id1),
        },
    )
    .await
    .result
    .unwrap();

    let checkpoint1_ = state::Checkpoints1Data::new(None, project_hash1);
    let checkpoint1 = client
        .get_checkpoint(checkpoint_id1)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(checkpoint1, checkpoint1_);

    let checkpoint2_ = state::Checkpoints1Data::new(Some(checkpoint_id1), project_hash2);
    let checkpoint2 = client
        .get_checkpoint(checkpoint_id2)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(checkpoint2, checkpoint2_);
}

#[async_std::test]
async fn create_checkpoint_without_parent() {
    let (client, _) = Client::new_emulator();
    let alice = key_pair_with_funds(&client).await;

    let project_hash = H256::random();
    let previous_checkpoint_id = Some(CheckpointId::random());

    let tx_included = submit_ok(
        &client,
        &alice,
        message::CreateCheckpoint {
            project_hash,
            previous_checkpoint_id,
        },
    )
    .await;

    assert_eq!(
        tx_included.result,
        Err(RegistryError::InexistentCheckpointId.into())
    )
}
