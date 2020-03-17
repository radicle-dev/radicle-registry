//! Register a project on the ledger
use std::convert::TryFrom;

use radicle_registry_client::*;

#[async_std::main]
async fn main() -> Result<(), Error> {
    env_logger::init();
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();

    let node_host = url::Host::parse("127.0.0.1").unwrap();
    let client = Client::create_with_executor(node_host).await?;

    let project_name = ProjectName::try_from("radicle-registry").unwrap();
    let org_id = OrgId::try_from("monadic").unwrap();

    // Choose some random project hash and create a checkpoint
    let project_hash = H256::random();
    let checkpoint_id = client
        .sign_and_submit_message(
            &alice,
            message::CreateCheckpoint {
                project_hash,
                previous_checkpoint_id: None,
            },
            346,
        )
        .await?
        .await?
        .result
        .unwrap();

    // Register the project
    client
        .sign_and_submit_message(
            &alice,
            message::RegisterProject {
                project_name: project_name.clone(),
                org_id: org_id.clone(),
                checkpoint_id,
                metadata: Bytes128::random(),
            },
            567,
        )
        .await?
        .await?
        .result
        .unwrap();

    println!(
        "Successfully registered project {}.{}",
        project_name, org_id
    );
    Ok(())
}
