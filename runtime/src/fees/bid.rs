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

use crate::{fees::BASE_FEE, Balance};

use frame_support::traits::WithdrawReason;

/// Bid
///
/// A Bid is an offer defined by transaction authors for the
/// registry to process their transactions. The bid should cover
/// all mandatory fees. The remainder left after deducting the
/// mandatory fees is used as a tip, which will grant priority
/// to the transaction in question accordingly to its value.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bid(Balance);

impl Bid {
    /// Create a Bid with the given `bid`.
    /// Fail if `bid` is insufficient to cover the mandatory fees.
    pub fn new(bid: Balance) -> Option<Self> {
        if bid < BASE_FEE {
            return None;
        }
        Some(Self(bid))
    }

    pub fn value(&self) -> Balance {
        self.0
    }

    pub fn withdraw_reasons(&self) -> frame_support::traits::WithdrawReasons {
        WithdrawReason::TransactionPayment | WithdrawReason::Tip
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;

    #[test]
    fn invalid_bid_insufficient() {
        assert!(
            Bid::new(0).is_none(),
            "An empty bid should not be enough to cover the mandatory fees."
        );
    }

    #[test]
    fn valid_bid_just_enough() {
        assert!(
            Bid::new(BASE_FEE).is_some(),
            "Bidding the base fee should have been enough."
        );
    }

    #[test]
    fn valid_bid_random() {
        for _ in 0..50 {
            // Generate a random bid between 1 and 9999.
            let random_bid: Balance = rand::thread_rng().gen_range(1, 10000);
            let bid = Bid::new(random_bid).unwrap();
            assert_eq!(bid.value(), random_bid);
            assert_eq!(
                bid.withdraw_reasons(),
                WithdrawReason::TransactionPayment | WithdrawReason::Tip
            );
        }
    }
}
