use futures01::future::Future;
use futures03::compat::{Compat, Future01CompatExt};
use futures03::future::FutureExt;

use radicle_registry_client::{
    ed25519, Client, ClientT as _, CryptoPair as _, Error, Transaction, TransactionExtra,
    TransferParams,
};

fn main() {
    env_logger::init();
    let mut runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(Compat::new(go().boxed())).unwrap();
    runtime.shutdown_now().wait().unwrap();
}

async fn go() -> Result<(), Error> {
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
    let bob = ed25519::Pair::from_string("//Bob", None).unwrap();
    let client = Client::create().compat().await?;

    let account_nonce = client.account_nonce(&alice.public()).compat().await?;
    let transfer_tx = Transaction::new_signed(
        &alice,
        TransferParams {
            recipient: bob.public(),
            balance: 1000,
        },
        TransactionExtra {
            nonce: account_nonce,
            genesis_hash: client.genesis_hash(),
        },
    );

    client.submit_transaction(transfer_tx).compat().await?;
    Ok(())
}
