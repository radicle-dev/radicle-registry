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

//! Provides [Emulator] backend to run the registry ledger in memory.

use futures::future::BoxFuture;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use sp_runtime::{traits::Block as _, traits::Hash as _, BuildStorage as _, Digest};
use sp_state_machine::backend::Backend as _;

use radicle_registry_runtime::{
    event,
    genesis::{BalancesConfig, GenesisConfig},
    registry, runtime_api, AccountId, Block, Hash, Hashing, Header, Runtime, RuntimeVersion,
};

use crate::backend;
use crate::interface::*;

/// [backend::Backend] implementation using native runtime code and in memory state through
/// [sp_io::TestExternalities] to emulate the ledger.
///
/// The clone of an [Emulator] will share the state with the original emulator using a [Mutex].
///
/// # Differences with real backend
///
/// * Every [backend::Backend::submit] call creates a new block that only contains the submited
///   transaction.
///
/// * The responses returned from the client never result in an [Error].
///
/// * The block author is fixed to [BLOCK_AUTHOR].
#[derive(Clone)]
pub struct Emulator {
    genesis_hash: Hash,
    inherent_data_providers: sp_inherents::InherentDataProviders,
    state: Arc<Mutex<EmulatorState>>,
}

/// Control handle to manipulate the state of [Emulator].
///
/// Construct this with [Emulator::control].
///
/// Cloning [EmulatorControl] will create another handle to manipulate the same emulator.
#[derive(Clone)]
pub struct EmulatorControl(Emulator);

impl EmulatorControl {
    /// Adds `count` number of empty blocks to the emulator chain.
    ///
    /// ```
    /// # #[async_std::main]
    /// # async fn main () {
    /// # use radicle_registry_client::{Client, ClientT};
    /// let (client, emulator) = Client::new_emulator();
    /// let header1 = client.block_header_best_chain().await.unwrap();
    /// emulator.add_blocks(3);
    /// let header2 = client.block_header_best_chain().await.unwrap();
    /// assert_eq!(header2.number, header1.number + 3)
    /// # }
    /// ```
    pub fn add_blocks(&self, count: u32) {
        for _ in 0..count {
            self.0.add_block(vec![]);
        }
    }
}

/// Mutable state of the emulator.
struct EmulatorState {
    test_ext: sp_io::TestExternalities,
    tip_header: Header,
    headers: HashMap<BlockHash, Header>,
}

/// Block author account used when the emulator creates blocks.
pub const BLOCK_AUTHOR: AccountId = ed25519::Public([0u8; 32]);

impl Emulator {
    pub fn new() -> Self {
        let genesis_config = make_genesis_config();
        let mut test_ext = sp_io::TestExternalities::new(genesis_config.build_storage().unwrap());
        let genesis_hash = init_runtime(&mut test_ext);

        let registry_inherent_data = registry::AuthoringInherentData {
            block_author: BLOCK_AUTHOR,
        };

        let inherent_data_providers = sp_inherents::InherentDataProviders::new();

        // Can only fail if a provider with the same name is already registered.
        inherent_data_providers
            .register_provider(sp_timestamp::InherentDataProvider)
            .unwrap();
        inherent_data_providers
            .register_provider(registry_inherent_data)
            .unwrap();

        let tip_header = Header {
            parent_hash: Hash::zero(),
            number: 1,
            state_root: Hash::zero(),
            extrinsics_root: Hash::zero(),
            digest: Digest::default(),
        };
        let mut headers = HashMap::new();
        headers.insert(tip_header.hash(), tip_header.clone());

        Emulator {
            genesis_hash,
            inherent_data_providers,
            state: Arc::new(Mutex::new(EmulatorState {
                test_ext,
                tip_header,
                headers,
            })),
        }
    }

    pub fn control(&self) -> EmulatorControl {
        EmulatorControl(self.clone())
    }

    /// Add a block with `extrinsics` to the chain. Returns the added block and a list of events
    /// recorded during the execution of the block.
    fn add_block(
        &self,
        extrinsics: Vec<backend::UncheckedExtrinsic>,
    ) -> (Block, Vec<event::Record>) {
        let mut state = self.state.lock().unwrap();

        let new_tip_header_init = Header {
            parent_hash: state.tip_header.hash(),
            number: state.tip_header.number + 1,
            ..state.tip_header.clone()
        };

        let (block, event_records) = state.test_ext.execute_with(move || {
            runtime_api::initialize_block(&new_tip_header_init);

            let inherent_data = self.inherent_data_providers.create_inherent_data().unwrap();
            let inherents = runtime_api::inherent_extrinsics(inherent_data);
            let extrinsics = [inherents, extrinsics].concat();

            for extrinsic in &extrinsics {
                let _apply_result = runtime_api::apply_extrinsic(extrinsic.clone()).unwrap();
            }

            let header = runtime_api::finalize_block();
            let event_records = frame_system::Module::<Runtime>::events();

            (Block { header, extrinsics }, event_records)
        });

        state.tip_header = block.header.clone();
        state.headers.insert(block.hash(), block.header.clone());

        (block, event_records)
    }
}

#[async_trait::async_trait]
impl backend::Backend for Emulator {
    async fn submit(
        &self,
        extrinsic: backend::UncheckedExtrinsic,
    ) -> Result<BoxFuture<'static, Result<backend::TransactionIncluded, Error>>, Error> {
        let tx_hash = Hashing::hash_of(&extrinsic);
        let (block, event_records) = self.add_block(vec![extrinsic]);
        let event_records = event_records
            .into_iter()
            .map(crate::event::EventRecord::V2)
            .collect();

        let events =
            crate::backend::remote_node::extract_transaction_events(tx_hash, &block, event_records)
                .unwrap();

        Ok(Box::pin(futures::future::ready(Ok(
            backend::TransactionIncluded {
                tx_hash,
                block: block.hash(),
                events,
            },
        ))))
    }

    async fn fetch(
        &self,
        key: &[u8],
        block_hash: Option<BlockHash>,
    ) -> Result<Option<Vec<u8>>, Error> {
        if block_hash.is_some() {
            panic!("Passing a block hash 'fetch' for the client emulator is not supported")
        }

        let mut state = self.state.lock().unwrap();
        let maybe_data = state.test_ext.execute_with(|| sp_io::storage::get(key));
        Ok(maybe_data)
    }

    async fn fetch_keys(
        &self,
        prefix: &[u8],
        block_hash: Option<BlockHash>,
    ) -> Result<Vec<Vec<u8>>, Error> {
        if block_hash.is_some() {
            panic!("Passing a block hash 'fetch_keys' for the client emulator is not supported")
        }

        let state = self.state.lock().unwrap();
        let backend = state.test_ext.commit_all();

        let mut keys = Vec::new();
        backend.for_keys_with_prefix(prefix, |key| keys.push(Vec::from(key)));
        Ok(keys)
    }

    async fn block_header(
        &self,
        block_hash_opt: Option<BlockHash>,
    ) -> Result<Option<BlockHeader>, Error> {
        let state = self.state.lock().unwrap();
        let block_hash = match block_hash_opt {
            Some(block_hash) => block_hash,
            None => return Ok(Some(state.tip_header.clone())),
        };
        Ok(state.headers.get(&block_hash).cloned())
    }

    fn get_genesis_hash(&self) -> Hash {
        self.genesis_hash
    }

    async fn runtime_version(&self) -> Result<RuntimeVersion, Error> {
        Ok(radicle_registry_runtime::VERSION)
    }
}

/// Create [GenesisConfig] for the emulated chain.
///
/// Initializes the balance of the `//Alice` account with `2^60` tokens.
fn make_genesis_config() -> GenesisConfig {
    GenesisConfig {
        pallet_balances: Some(BalancesConfig {
            balances: vec![(
                ed25519::Pair::from_string("//Alice", None)
                    .unwrap()
                    .public(),
                1 << 60,
            )],
        }),
        pallet_sudo: None,
        system: None,
    }
}

/// Initialize the runtime state so that it is usable and return the genesis hash.
fn init_runtime(test_ext: &mut sp_io::TestExternalities) -> Hash {
    test_ext.execute_with(|| {
        // Insert the genesis block (number `1`) into the system. We don’t care about the
        // other parameters, they are not relevant.
        frame_system::Module::<Runtime>::initialize(
            &1,
            &[0u8; 32].into(),
            &[0u8; 32].into(),
            &Default::default(),
            frame_system::InitKind::Full,
        );
        // Now we can retrieve the block hash. But here the block number is zero-based.
        frame_system::Module::<Runtime>::block_hash(0)
    })
}
