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
//! It's capped at `2^192 - 1`. This limit guarantees that Substrate can calculate a sum
//! of difficulties of blocks in a chain of up to `2^64` height and not overflow. This cap
//! should never affect the difficulty with current technology due to constraints of the universe.
//!
//! In order to check if the data hash passes the difficulty test, it must be interpreted as
//! a big-endian 256-bit number. If it's smaller than or equal to the threshold, it passes.
//! The threshold is calculated from difficulty as `U256::max_value / difficulty`.

use crate::blockchain::{Block, Hash, Header};
use crate::pow::{harmonic_mean::HarmonicMean, Difficulty};
use radicle_registry_runtime::timestamp_in_digest;
use sc_client_api::{blockchain, AuxStore};
use sc_consensus_pow::{Error, PowAlgorithm, PowAux};
use sp_api::ProvideRuntimeApi;
use sp_consensus_pow::Seal;
use sp_core::{H256, U256};
use sp_runtime::traits::Header as _;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

type BlockId = sp_runtime::generic::BlockId<Block>;
type Result<T> = std::result::Result<T, Error<Block>>;
type Threshold = U256;

const NONCES_PER_MINING_ROUND: usize = 10_000_000;
const INITIAL_DIFFICULTY: u64 = 1_000_000;
const ADJUST_DIFFICULTY_DAMPING: u32 = 1;
const ADJUST_DIFFICULTY_CLAMPING: u32 = 200;
const ADJUST_DIFFICULTY_WINDOW_SIZE: u64 = 12;
const TARGET_BLOCK_TIME_MS: u64 = 60_000;
const TARGET_WINDOW_TIME_MS: u64 = ADJUST_DIFFICULTY_WINDOW_SIZE * TARGET_BLOCK_TIME_MS;

/// An implementation of the Blake3 PoW algorithm.
///
/// For more information about this PoW algorithm see the [module](index.html) documentation.
#[derive(Clone, Debug)]
pub struct Blake3Pow<C> {
    client: C,
    next_nonce: Arc<AtomicU64>,
}

impl<C> Blake3Pow<C> {
    /// Creates Blake3Pow with a random seed for generating nonces
    pub fn new(client: C) -> Self {
        Self::new_with_seed(client, rand::random())
    }

    /// Creates Blake3Pow with the specific seed for generating nonces
    pub fn new_with_seed(client: C, nonce_seed: u64) -> Self {
        let next_nonce = Arc::new(AtomicU64::new(nonce_seed));
        Blake3Pow { client, next_nonce }
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

impl<C> PowAlgorithm<Block> for Blake3Pow<Arc<C>>
where
    C: ProvideRuntimeApi<Block>,
    C: AuxStore,
    C: blockchain::HeaderBackend<Block>,
{
    type Difficulty = Difficulty;

    fn difficulty(&self, parent: Hash) -> Result<Self::Difficulty> {
        let mut prev_header = self.header(parent)?;
        if (*prev_header.number() as u64) <= ADJUST_DIFFICULTY_WINDOW_SIZE {
            return Ok(Difficulty::from(INITIAL_DIFFICULTY));
        }
        let mut difficulty_mean = HarmonicMean::new();
        for _ in 0..ADJUST_DIFFICULTY_WINDOW_SIZE {
            let difficulty = self.block_difficulty(prev_header.hash())?;
            difficulty_mean.push(difficulty);
            prev_header = self.header(*prev_header.parent_hash())?;
        }
        let avg_difficulty = difficulty_mean.calculate();
        let time_observed = self.window_mining_time_ms(prev_header.hash(), parent)?;
        Ok(next_difficulty(avg_difficulty, time_observed))
    }

    fn verify(
        &self,
        _parent: &BlockId,
        pre_hash: &Hash,
        seal: &Seal,
        difficulty: Self::Difficulty,
    ) -> Result<bool> {
        let mut verifier = NonceVerifier::new(pre_hash, difficulty);
        Ok(verifier.is_nonce_valid(&seal))
    }

    fn mine(
        &self,
        _parent: &BlockId,
        pre_hash: &Hash,
        difficulty: Self::Difficulty,
        _round: u32,
    ) -> Result<Option<Seal>> {
        let mut verifier = NonceVerifier::new(pre_hash, difficulty);
        for nonce in self.nonces_for_mining_round() {
            if verifier.is_nonce_valid(&nonce) {
                return Ok(Some(nonce.to_vec()));
            }
        }
        Ok(None)
    }
}

impl<C> Blake3Pow<Arc<C>>
where
    C: ProvideRuntimeApi<Block>,
    C: AuxStore,
    C: blockchain::HeaderBackend<Block>,
{
    fn header(&self, block_hash: Hash) -> Result<Header> {
        self.client
            .header(BlockId::hash(block_hash))
            .and_then(|num_opt| {
                num_opt.ok_or_else(|| {
                    sp_blockchain::Error::UnknownBlock(format!(
                        "Can't find a block for the hash {}",
                        block_hash
                    ))
                })
            })
            .map_err(Error::Client)
    }

    fn block_difficulty(&self, block_hash: H256) -> Result<Difficulty> {
        PowAux::read(&*self.client, &block_hash).map(|difficulty| difficulty.difficulty)
    }

    /// Calculates time it took to mine the blocks in the window.
    ///
    /// It accepts the last block in the window and the **parent** of the first block.
    /// This is necessary, because in order to obtain a block mining time one must calculate
    /// the difference between timestamps of the block and it's parent.
    fn window_mining_time_ms(
        &self,
        first_block_parent_hash: Hash,
        last_block_hash: Hash,
    ) -> Result<u64> {
        let start = self.block_timestamp_ms(first_block_parent_hash)?;
        let end = self.block_timestamp_ms(last_block_hash)?;
        Ok(end - start)
    }

    fn block_timestamp_ms(&self, block_hash: Hash) -> Result<u64> {
        let header = self.header(block_hash)?;
        timestamp_in_digest::load(&header.digest)
            .ok_or_else(|| Error::Runtime("Timestamp not set in digest".into()))?
            .map_err(Error::Codec)
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

/// Calculates the difficulty for the next block based on the window of the previous blocks
///
/// `avg` - the average difficulty of the blocks in the window
/// `time_observed` - the total time it took to create the blocks in the window
fn next_difficulty(avg: Difficulty, time_observed: u64) -> Difficulty {
    // This won't overflow, because difficulty is capped at using only its low 192 bits
    let new_raw = avg * TARGET_WINDOW_TIME_MS / time_observed.max(1);
    if new_raw > avg {
        let delta = new_raw - avg;
        let damped_delta = delta / ADJUST_DIFFICULTY_DAMPING;
        let new_damped = avg + damped_delta;
        let new_max = avg * ADJUST_DIFFICULTY_CLAMPING;
        new_damped.min(new_max).min(max_difficulty())
    } else {
        let delta = avg - new_raw;
        let damped_delta = delta / ADJUST_DIFFICULTY_DAMPING;
        let new_damped = avg - damped_delta;
        // Clamping matters only when ADJUST_DIFFICULTY_CLAMPING > ADJUST_DIFFICULTY_DAMPING
        let new_min = avg / ADJUST_DIFFICULTY_CLAMPING;
        new_damped.max(new_min)
    }
}

// This should be a constant when it becomes possible
fn max_difficulty() -> U256 {
    U256::MAX >> 64
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn next_difficulty_tests() {
        assert_next_difficulty(200, 0);
        assert_next_difficulty(200, 1);
        assert_next_difficulty(200, 25);
        assert_next_difficulty(194, 26);
        assert_next_difficulty(133, 50);
        assert_next_difficulty(100, 100);
        assert_next_difficulty(84, 200);
        assert_next_difficulty(68, 5000);
        assert_next_difficulty(67, 5001);
        assert_next_difficulty(67, 10000);
    }

    // assume that the average window difficulty is 100 and the target window time is 100
    fn assert_next_difficulty(expected: u64, time_observed: u64) {
        let adjusted_time_observed = TARGET_WINDOW_TIME_MS * time_observed / 100;
        let actual = next_difficulty(U256::from(100), adjusted_time_observed);
        assert_eq!(
            U256::from(expected),
            actual,
            "Failed for time_observed {}",
            time_observed
        );
    }

    #[test]
    fn dunno_lol() {
        // fn one_to_one((_idx, diff): (usize, &U256)) -> u64 {
        //     diff.low_u64()
        // }

        // fn fast_oscilate_1000((idx, diff): (usize, &U256)) -> u64 {
        //     let time = diff.low_u64();
        //     match (idx / 40) % 2 {
        //         0 => time * 100 / 120,
        //         _ => time * 100 / 80,
        //     }
        // }

        // test_difficulty(30000, 1000);
        // test_difficulty(90000, 1000);
        // test_difficulty(30000, 1000);
        test_difficulty(6000, 1500);
        panic!();
    }

    // hash rate: 100h/s
    fn test_difficulty(initial: u64, cycles: usize) {
        const WINDOW: usize = ADJUST_DIFFICULTY_WINDOW_SIZE as usize;
        // difficulty, time in ms it took to mine
        let mut history: Vec<(U256, u64)> = vec![];
        let mut random = rand::thread_rng();
        let mut low = 0;
        let mut high = 0;
        while history.len() < cycles {
            let difficulty = if history.len() < WINDOW {
                U256::from(initial)
            } else {
                let mut mean = HarmonicMean::new();
                let mut window_time = 0;
                for (diff, time) in history.iter().rev().take(WINDOW) {
                    mean.push((*diff).into());
                    window_time += time;
                }
                let diff_mean = mean.calculate();
                let next = next_difficulty(diff_mean, window_time);
                println!("WINDOW TIME {:.3} DIFF MEAN {:.3} NEXT {:.3}",
                    window_time as f64 / TARGET_WINDOW_TIME_MS as f64,
                    diff_mean.low_u128() as f64 / 6000.,
                    next.low_u128() as f64 / diff_mean.low_u128() as f64);
                if window_time < TARGET_WINDOW_TIME_MS {
                    low += 1
                } else {
                    high += 1
                }
                next
            };
            let bernoulli = rand::distributions::Bernoulli::new(1f64 / difficulty.low_u128() as f64).unwrap();
            let mut time = 10;
            while !rand::distributions::Distribution::sample(&bernoulli, &mut random) {
                time += 10;
            }
            history.push((difficulty, time));
        }
        println!("\nINITIAL DIFF {}", initial);
        println!("AVG DIFF {}", history.iter().map(|(d, _)| d.low_u64()).sum::<u64>() as usize / history.len());
        println!("AVG TIME {}", history.iter().map(|(_, t)| t).sum::<u64>() as usize / history.len());
        println!("FINAL DIFF {}", history.last().unwrap().0);
        println!("LOW {} HIGH {}", low, high);
        println!("RATE {}", history.len() as f64 / history.iter().map(|(_, t)| t).sum::<u64>() as f64 * 3600000.)
        // for (idx, item) in history.iter().enumerate() {
        //     println!("{:04}; {}", idx + 1, item);
        // }

    }
}
