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

//! PoW algorithm implementation based on Blake3 hashing
//!
//! The difficulty is an average number of hashes that need to be checked in order mine a block.
//! It must be capped at `2^192 - 1`. This limit guarantees that Substrate can calculate a sum
//! of difficulties of blocks in a chain of up to `2^64` height and not overflow. This cap
//! should be impossible to break with current technology due to constraints of the universe.
//!
//! In order to check if the data hash passes the difficulty test, it must be interpreted as
//! a big-endian 256-bit number. If it's smaller than or equal to the threshold, it passes.
//! The threshold is calculated from difficulty as `U256::max_value / difficulty`.
//!
//! There's no difficulty adjustment algorithm yet.

use radicle_registry_runtime::opaque::Block;
use radicle_registry_runtime::Hash;
use sc_consensus_pow::{Error, PowAlgorithm};
use sp_consensus_pow::Seal;
use sp_core::uint::U256;
use sp_runtime::generic::BlockId;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

type Difficulty = U256;
type Threshold = U256;

const NONCES_PER_MINING_ROUND: usize = 10_000_000;

/// An implementation of the Blake3 PoW algorithm.
///
/// For more information about this PoW algorithm see the [module](index.html) documentation.
#[derive(Clone)]
pub struct Blake3Pow {
    next_nonce: Arc<AtomicU64>,
}

impl Blake3Pow {
    /// Creates Blake3Pow with a random seed for generating nonces
    pub fn new() -> Self {
        Self::new_with_seed(rand::random())
    }

    /// Creates Blake3Pow with the specific seed for generating nonces
    pub fn new_with_seed(nonce_seed: u64) -> Self {
        let next_nonce = Arc::new(AtomicU64::new(nonce_seed));
        Blake3Pow { next_nonce }
    }

    fn nonces_for_mining_round(&self) -> impl Iterator<Item = [u8; 8]> {
        let first_nonce = self
            .next_nonce
            // fetch_add wraps on overflow
            .fetch_add(NONCES_PER_MINING_ROUND as u64, Ordering::Relaxed);
        std::iter::successors(Some(first_nonce), |prev_nonce| {
            Some(prev_nonce.wrapping_add(1))
        })
        .take(NONCES_PER_MINING_ROUND)
        .map(u64::to_ne_bytes)
    }
}

impl PowAlgorithm<Block> for Blake3Pow {
    type Difficulty = Difficulty;

    fn difficulty(&self, _parent: &BlockId<Block>) -> Result<Self::Difficulty, Error<Block>> {
        // To be enforced: only 192 lower bits can be set, see module docs
        Ok(Difficulty::from(10_000_000))
    }

    fn verify(
        &self,
        _parent: &BlockId<Block>,
        pre_hash: &Hash,
        seal: &Seal,
        difficulty: Self::Difficulty,
    ) -> Result<bool, Error<Block>> {
        let mut verifier = NonceVerifier::new(pre_hash, difficulty);
        Ok(verifier.is_nonce_valid(&seal))
    }

    fn mine(
        &self,
        _parent: &BlockId<Block>,
        pre_hash: &Hash,
        difficulty: Self::Difficulty,
        _round: u32,
    ) -> Result<Option<Seal>, Error<Block>> {
        let mut verifier = NonceVerifier::new(pre_hash, difficulty);
        for nonce in self.nonces_for_mining_round() {
            if verifier.is_nonce_valid(&nonce) {
                return Ok(Some(nonce.to_vec()));
            }
        }
        Ok(None)
    }
}

struct NonceVerifier {
    payload: Vec<u8>,
    threshold: Threshold,
}

impl NonceVerifier {
    fn new(pre_hash: &Hash, difficulty: Difficulty) -> Self {
        NonceVerifier {
            payload: pre_hash.as_bytes().to_vec(),
            threshold: difficulty_to_threshold(difficulty),
        }
    }

    fn is_nonce_valid(&mut self, nonce: &[u8]) -> bool {
        let original_payload_len = self.payload.len();
        self.payload.extend_from_slice(nonce);
        let hash = blake3::hash(&self.payload);
        self.payload.truncate(original_payload_len);
        hash_passes_threshold_test(hash, self.threshold)
    }
}

fn difficulty_to_threshold(difficulty: Difficulty) -> Threshold {
    Threshold::max_value() / difficulty
}

fn hash_passes_threshold_test(hash: blake3::Hash, threshold: Threshold) -> bool {
    let hash_value = Threshold::from_big_endian(hash.as_bytes());
    hash_value <= threshold
}
