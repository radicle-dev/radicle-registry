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
    let project_id = (
        ProjectName::from_string("NAME".to_string()).unwrap(),
        ProjectDomain::from_string("DOMAIN".to_string()).unwrap(),
    );
    client
        .sign_and_submit_call(
            &alice,
            RegisterProjectParams {
                id: project_id.clone(),
                description: "DESCRIPTION".to_string(),
                img_url: "IMG_URL".to_string(),
                checkpoint_id,
            },
        )
        .compat()
        .await?
        .compat()
        .await?;
    println!("{:?}", project_id);
    Ok(())
}
