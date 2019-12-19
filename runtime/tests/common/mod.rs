/// Miscellaneous helpers used throughout other tests.
use futures::prelude::*;
use rand::distributions::Alphanumeric;
use rand::Rng;

use radicle_registry_client::*;

#[allow(dead_code)]
pub fn create_project_with_checkpoint(client: &Client, author: &ed25519::Pair) -> Project {
    let checkpoint_id = client
        .create_checkpoint(&author, H256::random(), None)
        .wait()
        .unwrap();

    let params = random_register_project_params(checkpoint_id);

    client
        .register_project(&author, params.clone())
        .wait()
        .unwrap();

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
        .collect::<String>()
        .into_bytes();
    let domain = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(32)
        .collect::<String>()
        .into_bytes();
    let id = (name, domain);

    let description = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(50)
        .collect::<String>()
        .into_bytes();
    let img_url = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(50)
        .collect::<String>()
        .into_bytes();

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
