//! Register a project on the ledger
use futures01::future::Future;
use futures03::compat::{Compat, Future01CompatExt};
use futures03::future::FutureExt;

use radicle_registry_client::*;

fn main() {
    env_logger::init();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(Compat::new(go().boxed())).unwrap();
    runtime.shutdown_now().wait().unwrap();
}

async fn go() -> Result<(), Error> {
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create(node_host).compat().await?;

    let project_name = ProjectName::from_string("radicle-registry".to_string()).unwrap();
    let project_domain = ProjectDomain::from_string("rad".to_string()).unwrap();
    let project_id = (project_name.clone(), project_domain.clone());

    // Choose some random project hash and create a checkpoint
    let project_hash = H256::random();
    let checkpoint_id = client
        .sign_and_submit_call(
            &alice,
            CreateCheckpointParams {
                project_hash,
                previous_checkpoint_id: None,
            },
        )
        .compat()
        .await?
        .compat()
        .await?
        .result
        .unwrap();

    // Register the project
    client
        .sign_and_submit_call(
            &alice,
            RegisterProjectParams {
                id: project_id.clone(),
                description: String::default(),
                img_url: String::default(),
                checkpoint_id,
            },
        )
        .compat()
        .await?
        .compat()
        .await?
        .result
        .unwrap();

    println!(
        "Successfully registered project {}.{}",
        project_name, project_domain
    );
    Ok(())
}
