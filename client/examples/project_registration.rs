// Radicle Registry
// Copyright (C) 2019 Monadic GmbH <radicle@monadic.xyz>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License version 3 as
// published by the Free Software Foundation.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

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
    let org_id = Id::try_from("monadic").unwrap();

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
                project_domain: ProjectDomain::Org(org_id.clone()),
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
