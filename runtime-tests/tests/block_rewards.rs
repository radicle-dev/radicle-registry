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
use radicle_registry_runtime::registry::BLOCK_REWARD;
use radicle_registry_test_utils::*;
use sp_runtime::Permill;

/// Assert that block rewards and transaction fees are credited to the block author.
#[async_std::test]
async fn block_rewards_credited() {
    let (client, _) = Client::new_emulator();

    let alice = key_pair_from_string("Alice");
    let bob = key_pair_from_string("Bob").public();

    let fee = 3000;
    submit_ok_with_fee(
        &client,
        &alice,
        message::Transfer {
            recipient: bob,
            balance: 1000,
        },
        fee,
    )
    .await;

    let rewards = client.free_balance(&EMULATOR_BLOCK_AUTHOR).await.unwrap();
    let fee_reward = Permill::from_percent(99) * fee;
    assert_eq!(rewards, fee_reward + BLOCK_REWARD);
}
