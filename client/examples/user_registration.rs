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

//! Register a user on the ledger.
use sp_core::crypto::Pair;
use std::convert::TryFrom;

use radicle_registry_client::{ed25519, message, Client, ClientT, Id};

#[async_std::main]
async fn main() {
    env_logger::init();
    let client = {
        let node_host = url::Host::parse("127.0.0.1").unwrap();
        Client::create_with_executor(node_host).await.unwrap()
    };
    let alice = ed25519::Pair::from_string("//Alice", None).unwrap();
    let user_id = Id::try_from("cloudhead").unwrap();

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
