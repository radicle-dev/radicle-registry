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

use crate::{Balance};
use crate::fees::{BaseFee, Fee, Tip};
use frame_support::dispatch::DispatchError;

/// Bid
///
/// A Bid is an offer defined by a transaction author for the
/// registry to process that transaction. The bid should cover
/// all mandatory fees. The remainder left after deducting the
/// mandatory fees is used as a tip, which will grant priority
/// to the transaction in question accordingly.
pub struct Bid {
    base_fee: BaseFee,
    tip: Tip
}

impl Bid {
    /// Create a Bid with the given `bid`.
    /// Fail when `bid` is insufficient to cover all the
    /// mandatory fees, now being the `base_fee` alone.
    pub fn new(bid: Balance) -> Result<Self, InvalidBid> {
        let base_fee = BaseFee{};
        let base_fee_value = base_fee.value();
        if bid < base_fee_value {
            return Err(InvalidBid::Insufficient)
        }
        Ok(
            Self {
                base_fee,
                tip: Tip(bid - base_fee_value),
            }
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum InvalidBid {
    /// The bid is insufficient to cover all mandatory costs.
    Insufficient,
}

impl From<InvalidBid> for DispatchError {
    fn from(error: InvalidBid) -> Self {
        DispatchError::Module {
            index: 42,
            error: error as u8,
            message: None,
        }
    }
}
