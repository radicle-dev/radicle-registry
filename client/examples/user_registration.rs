//! Register a user on the ledger.
use sp_core::crypto::Pair;
use std::convert::TryFrom;

use radicle_registry_client::{ed25519, message, Client, ClientT, UserId};

#[async_std::main]
async fn main() {
    env_logger::init();
    let client = {
        let node_host = url::Host::parse("127.0.0.1").unwrap();
        Client::create_with_executor(node_host).await.unwrap()
    };
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
    let user_id = UserId::try_from("cloudhead").unwrap();

    // Register the user.
    client
        .sign_and_submit_message(
            &alice,
            message::RegisterUser {
                user_id: user_id.clone(),
            },
            100,
        )
        .await
        .unwrap()
        .await
        .unwrap()
        .result
        .unwrap();

    println!("Successfully registered user {}", user_id);
}
