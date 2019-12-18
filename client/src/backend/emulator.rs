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

use std::sync::{Arc, Mutex};

use sr_primitives::{traits::Hash as _, BuildStorage as _};

use radicle_registry_runtime::{BalancesConfig, Executive, GenesisConfig, Hash, Hashing, Runtime};

use crate::backend;
use crate::interface::*;

/// [backend::Backend] implementation using native runtime code and in memory state through
/// [sr_io::TestExternalities] to emulate the ledger.
///
/// # Differences with real [Client]
///
/// The [Emulator] does not produce blocks. In particular the `blocks` field in
/// [TransactionApplied]` always equals `Hash::default` when returned from [Client::submit].
///
/// The responses returned from the client never result in an [Error].
#[derive(Clone)]
pub struct Emulator {
    test_ext: Arc<Mutex<sr_io::TestExternalities>>,
    genesis_hash: Hash,
}

impl Emulator {
    pub fn new() -> Self {
        let genesis_config = make_genesis_config();
        let mut test_ext = sr_io::TestExternalities::new(genesis_config.build_storage().unwrap());
        let genesis_hash = init_runtime(&mut test_ext);
        Emulator {
            test_ext: Arc::new(Mutex::new(test_ext)),
            genesis_hash,
        }
    }
}

#[async_trait::async_trait]
impl backend::Backend for Emulator {
    async fn submit(
        &self,
        extrinsic: backend::UncheckedExtrinsic,
    ) -> Result<backend::TransactionApplied, Error> {
        let tx_hash = Hashing::hash_of(&extrinsic);
        let test_ext = &mut self.test_ext.lock().unwrap();
        let events = test_ext.execute_with(move || {
            let event_start_index = paint_system::Module::<Runtime>::event_count();
            // We ignore the dispatch result. It is provided through the system event
            // TODO Pass on apply errors instead of unwrapping.
            let _dispatch_result = Executive::apply_extrinsic(extrinsic).unwrap();
            paint_system::Module::<Runtime>::events()
                .into_iter()
                .skip(event_start_index as usize)
                .map(|event_record| event_record.event)
                .collect::<Vec<Event>>()
        });
        Ok(backend::TransactionApplied {
            tx_hash,
            block: Default::default(),
            events,
        })
    }

    async fn fetch(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Error> {
        let test_ext = &mut self.test_ext.lock().unwrap();
        let maybe_data = test_ext.execute_with(|| sr_io::storage::get(key));
        Ok(maybe_data)
    }

    fn get_genesis_hash(&self) -> Hash {
        self.genesis_hash
    }
}

/// Create [GenesisConfig] for the emulated chain.
///
/// Initializes the balance of the `//Alice` account with `2^60` tokens.
fn make_genesis_config() -> GenesisConfig {
    GenesisConfig {
        paint_aura: None,
        paint_balances: Some(BalancesConfig {
            balances: vec![(
                ed25519::Pair::from_string("//Alice", None)
                    .unwrap()
                    .public(),
                1 << 60,
            )],
            vesting: vec![],
        }),
        paint_sudo: None,
        paint_grandpa: None,
        system: None,
    }
}

/// Initialize the runtime state so that it is usable and return the genesis hash.
fn init_runtime(test_ext: &mut sr_io::TestExternalities) -> Hash {
    test_ext.execute_with(|| {
        // Insert the genesis block (number `1`) into the system. We don’t care about the
        // other parameters, they are not relevant.
        paint_system::Module::<Runtime>::initialize(
            &1,
            &[0u8; 32].into(),
            &[0u8; 32].into(),
            &Default::default(),
        );
        // Now we can retrieve the block hash. But here the block number is zero-based.
        paint_system::Module::<Runtime>::block_hash(0)
    })
}
