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

use pow_consensus::{Error, PowAlgorithm};
use radicle_registry_runtime::opaque::Block;
use radicle_registry_runtime::Hash;
use sp_consensus_pow::Seal;
use sp_runtime::generic::BlockId;

#[derive(Clone)]
pub struct DummyPow;

impl PowAlgorithm<Block> for DummyPow {
    type Difficulty = u128;

    fn difficulty(&self, _parent: &BlockId<Block>) -> Result<Self::Difficulty, Error<Block>> {
        Ok(0)
    }

    fn verify(
        &self,
        _parent: &BlockId<Block>,
        _pre_hash: &Hash,
        _seal: &Seal,
        _difficulty: Self::Difficulty,
    ) -> Result<bool, Error<Block>> {
        Ok(true)
    }

    fn mine(
        &self,
        _parent: &BlockId<Block>,
        _pre_hash: &Hash,
        _difficulty: Self::Difficulty,
        _round: u32,
    ) -> Result<Option<Seal>, Error<Block>> {
        std::thread::sleep(std::time::Duration::from_secs(1));
        Ok(Some(vec![]))
    }
}
