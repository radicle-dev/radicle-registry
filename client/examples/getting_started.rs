//! Getting started with the client by transfering funds.
//!
//! We’re transferring some funds from Alice to Bob and will inspect the node state.
//!
//! To run this example you need a running dev node. You can start it with
//! `./scripts/run-dev-node`.
use futures::compat::Compat;
use futures::future::FutureExt;

use radicle_registry_client::*;

// Wrapper to run the async function `go`.
fn main() {
    env_logger::init();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    let result = runtime.block_on(Compat::new(go().boxed()));
    match result {
        Ok(()) => (),
        Err(err) => {
            println!("ERROR: {:?}", err);
            std::process::exit(1)
        }
    }
}

// We call [Future01CompatExt::compat] on all futures because the client only uses futures v0.1.
async fn go() -> Result<(), Error> {
    // Create a key pair to author transactions from some seed data. This account is initialized
    // with funds on the local development chain.
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
    println!("Sending funds from //Alice ({})", alice.public());

    // The receiver of the money transfer is Bob. We only need the public key
    let bob_public = ed25519::Pair::from_string("//Bob", None).unwrap().public();
    println!("Recipient: //Bob ({})", bob_public);

    // Create and connect to a client on local host
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    println!("Connecting to node on {}", node_host);
    let client = Client::create(node_host).await?;

    // Show balances of Alice’s and Bob’s accounts
    let balance_alice = client.free_balance(&alice.public()).await?;
    println!("Balance Alice: {}", balance_alice);
    let balance_bob = client.free_balance(&bob_public).await?;
    println!("Balance Bob:   {}", balance_bob);

    // Sign and submit the message. If successful, returns a future that
    // resolves when the transaction is included in a block.
    print!("Submitting transfer transaction... ");
    let transfer_submitted = client
        .sign_and_submit_message(
            &alice,
            message::Transfer {
                recipient: bob_public,
                balance: 1,
            },
            777,
        )
        .await?;
    println!("done");

    print!("Waiting for transaction to be included in block... ");
    let transfer_applied = transfer_submitted.await?;
    println!("done");

    // We can use the [TransactionApplied] struct to get the block.
    println!("Transaction included in block {}", transfer_applied.block);

    // We can also use it to get result of applying the transaction in the ledger. This might fail
    // for example if the transaction author does not have the necessary funds.
    match transfer_applied.result {
        Ok(()) => println!("Funds successfully transferred!"),
        Err(err) => println!("Failed to transfer funds: {:?}", err),
    }

    // Show the new balances
    let balance_alice = client.free_balance(&alice.public()).await?;
    println!("Balance Alice: {}", balance_alice);
    let balance_bob = client.free_balance(&bob_public).await?;
    println!("Balance Bob:   {}", balance_bob);

    Ok(())
}
