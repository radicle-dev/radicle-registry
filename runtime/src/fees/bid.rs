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

use crate::fees::{BaseFee, Fee, Tip};
use crate::Balance;

/// Bid
///
/// A Bid is an offer defined by transaction authors for the
/// registry to process their transactions. The bid should cover
/// all mandatory fees. The remainder left after deducting the
/// mandatory fees is used as a tip, which will grant priority
/// to the transaction in question accordingly to its value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bid {
    pub base_fee: BaseFee,
    pub tip: Tip,
}

impl Bid {
    /// Create a Bid with the given `bid`.
    /// Fail when `bid` is insufficient to cover all the
    /// mandatory fees, now being the `base_fee` alone.
    pub fn new(bid: Balance) -> Option<Self> {
        let base_fee = BaseFee;
        bid.checked_sub(base_fee.value()).map(|remainder| Self {
            base_fee,
            tip: Tip(remainder),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;

    #[test]
    fn invalid_bid_insufficient() {
        // The bid is insufficient to cover all mandatory fees.
        assert_eq!(Bid::new(0), None);
    }

    #[test]
    fn valid_bid_just_enough() {
        // The bid is just enough to cover the mandatory base fee,
        // leaving a tip of 0 left.
        let bid = Bid::new(BaseFee.value()).unwrap();
        assert_eq!(bid.base_fee.value(), 1);
        assert_eq!(bid.tip.value(), 0);
    }

    #[test]
    fn valid_bid_random() {
        for _ in 0..50 {
            // Generate a random bid between 1 and 9999.
            let random_bid: Balance = rand::thread_rng().gen_range(1, 10000);
            let bid = Bid::new(random_bid).unwrap();
            assert_eq!(bid.base_fee.value(), 1);
            assert_eq!(bid.tip.value(), random_bid - BaseFee.value())
        }
    }
}
