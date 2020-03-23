use radicle_registry_client::*;
use radicle_registry_runtime::registry::BLOCK_REWARD;
use radicle_registry_test_utils::*;
use sp_runtime::Permill;

/// Assert that block rewards and transaction fees are credited to the block author.
#[async_std::test]
async fn block_rewards_credited() {
    let client = Client::new_emulator();

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
