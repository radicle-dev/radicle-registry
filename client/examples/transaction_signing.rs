//! Offline signing and creation of a `Transfer` transaction.

use radicle_registry_client::*;

#[async_std::main]
async fn main() -> Result<(), Error> {
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
    let bob = ed25519::Pair::from_string("//Bob", None).unwrap();
    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await?;

    // Construct `TransactionExtra` data that is required to validate a transaction.
    let account_nonce = client.account_nonce(&alice.public()).await?;
    let transaction_extra = TransactionExtra {
        nonce: account_nonce,
        genesis_hash: client.genesis_hash(),
        fee: 10,
    };

    // Construct the transaction
    let transfer_tx = Transaction::new_signed(
        &alice,
        message::Transfer {
            recipient: bob.public(),
            balance: 1000,
        },
        transaction_extra,
    );

    client.submit_transaction(transfer_tx).await?.await?;
    Ok(())
}
