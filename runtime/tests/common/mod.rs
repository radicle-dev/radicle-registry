/// Miscellaneous helpers used throughout other tests.
use futures01::prelude::*;
use rand::distributions::Alphanumeric;
use rand::Rng;

use radicle_registry_client::*;

/// Submit a transaction and wait for it to be successfully applied.
///
/// Panics if submission errors.
#[allow(dead_code)]
pub fn submit_ok<Call_: Call>(
    client: &Client,
    author: &ed25519::Pair,
    call: Call_,
) -> TransactionApplied<Call_> {
    client
        .sign_and_submit_call(&author, call)
        .wait()
        .unwrap()
        .wait()
        .unwrap()
}

#[allow(dead_code)]
pub fn create_project_with_checkpoint(client: &Client, author: &ed25519::Pair) -> Project {
    let checkpoint_id = submit_ok(
        &client,
        &author,
        CreateCheckpointParams {
            project_hash: H256::random(),
            previous_checkpoint_id: None,
        },
    )
    .result
    .unwrap();

    let params = random_register_project_params(checkpoint_id);

    submit_ok(&client, &author, params.clone());

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
