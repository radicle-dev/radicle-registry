use futures01::future::Future;
use futures03::compat::{Compat, Future01CompatExt};
use futures03::future::FutureExt;

use radicle_registry_client::{
    ed25519, Client, ClientT as _, CryptoPair as _, Error, RegisterProjectParams, H256,
};

fn main() {
    env_logger::init();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(Compat::new(go().boxed())).unwrap();
    runtime.shutdown_now().wait().unwrap();
}

async fn go() -> Result<(), Error> {
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
    let client = Client::create().compat().await?;
    let project_hash = H256::random();
    let checkpoint_id = client
        .create_checkpoint(&alice, project_hash, None)
        .compat()
        .await?;
    let project_id = ("NAME".to_string().into_bytes(), "DOMAIN".to_string().into_bytes());
    client
        .register_project(
            &alice,
            RegisterProjectParams {
                id: project_id.clone(),
                description: "DESCRIPTION".to_string().into_bytes(),
                img_url: "IMG_URL".to_string().into_bytes(),
                checkpoint_id,
            },
        )
        .compat()
        .await?;
    println!("{:?}", project_id);
    Ok(())
}
