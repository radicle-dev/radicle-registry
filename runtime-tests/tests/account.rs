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

use radicle_registry_client::*;
use radicle_registry_test_utils::*;

/// Assert that a known account is recognized as existent on chain
#[async_std::test]
async fn account_exists() {
    let (client, _) = Client::new_emulator();
    let account_on_chain = key_pair_with_associated_user(&client).await.0.public();

    assert!(
        client.account_exists(&account_on_chain).await.unwrap(),
        "Random account shouldn't exist on chain"
    );
}

/// Assert that a random account id does not exist on chain
#[async_std::test]
async fn random_account_does_not_exist() {
    let (client, _) = Client::new_emulator();
    let random_account = ed25519::Pair::generate().0.public();

    assert!(
        !client.account_exists(&random_account).await.unwrap(),
        "Account was expected to be on chain"
    );
}
