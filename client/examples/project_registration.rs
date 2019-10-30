use futures::compat::{Compat, Future01CompatExt};
use futures::future::FutureExt;
use futures01::future::Future;

use radicle_registry_client::{ed25519, Client, Error, RegisterProjectParams};
use substrate_primitives::crypto::Pair;

fn main() {
    env_logger::init();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(Compat::new(go().boxed())).unwrap();
    runtime.shutdown_now().wait().unwrap();
}

async fn go() -> Result<(), Error> {
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
    let client = Client::create().compat().await?;
    let project_id = client
        .register_project(
            &alice,
            RegisterProjectParams {
                name: "NAME".to_string(),
                description: "DESCRIPTION".to_string(),
                img_url: "IMG_URL".to_string(),
            },
        )
        .compat()
        .await?;
    println!("{:?}", project_id);
    Ok(())
}
