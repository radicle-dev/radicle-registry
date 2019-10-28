//! Test the client against a running node.
//!
//! Note that chain state is shared between the test runs.

#![feature(async_closure)]
use futures::compat::{Compat, Future01CompatExt};
use futures::prelude::*;
use futures01::future::Future as _Future;

use radicle_registry_client::{ed25519, Client, Pair};

#[test]
fn counter() {
    run(async {
        let client = Client::create().compat().await.unwrap();
        let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

        client.counter_inc(&alice).compat().await.unwrap();
        client.counter_inc(&alice).compat().await.unwrap();

        let counter = client.get_counter().compat().await.unwrap().unwrap().0;
        assert_eq!(counter, 2);
    })
}

fn run(f: impl Future<Output = ()> + Send + 'static) {
    env_logger::init();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime
        .block_on(Compat::new(
            f.map(|()| -> Result<(), ()> { Ok(()) }).boxed(),
        ))
        .unwrap();
    runtime.shutdown_now().wait().unwrap();
}
