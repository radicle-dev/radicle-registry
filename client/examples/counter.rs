//! Increase the counter and read out the increased counter.
use futures::compat::{Compat, Future01CompatExt};
use futures::future::FutureExt;

use radicle_registry_client::{ed25519, Client, Error};
use substrate_primitives::crypto::Pair;

fn main() {
    env_logger::init();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(Compat::new(go().boxed())).unwrap();
}

async fn go() -> Result<(), Error> {
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
    let client = Client::create().compat().await?;
    client.counter_inc(&alice).compat().await?;
    let value = client.get_counter().compat().await?;
    println!("{:?}", value);
    Ok(())
}
