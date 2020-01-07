/// Miscellaneous helpers used throughout other tests.
use futures01::prelude::*;
use rand::distributions::Alphanumeric;
use rand::Rng;

use radicle_registry_client::*;

#[allow(dead_code)]
pub fn create_project_with_checkpoint(client: &Client, author: &ed25519::Pair) -> Project {
    let checkpoint_id = client
        .submit(
            &author,
            CreateCheckpointParams {
                project_hash: H256::random(),
                previous_checkpoint_id: None,
            },
        )
        .wait()
        .unwrap()
        .result
        .unwrap();

    let params = random_register_project_params(checkpoint_id);

    client.submit(&author, params.clone()).wait().unwrap();

    client.get_project(params.id).wait().unwrap().unwrap()
}

/// Create random parameters to register a project with.
/// The project's name and domain will be alphanumeric strings with 32
/// characters, and the description and image URL will be alphanumeric strings
/// with 50 characters.
pub fn random_register_project_params(checkpoint_id: CheckpointId) -> RegisterProjectParams {
    let name = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .collect::<String>();
    let domain = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .collect::<String>();
    let id = (name.parse().unwrap(), domain.parse().unwrap());

    let description = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(50)
        .collect::<String>();
    let img_url = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(50)
        .collect::<String>();

    RegisterProjectParams {
        id,
        description,
        img_url,
        checkpoint_id,
    }
}

#[allow(dead_code)]
pub fn key_pair_from_string(value: impl AsRef<str>) -> ed25519::Pair {
    ed25519::Pair::from_string(format!("//{}", value.as_ref()).as_str(), None).unwrap()
}
