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

//! Fee module
//!
//! This crate defines all things fees:
//! * the types of fees supported by the registry
//! * the [crate::fees::bid] module that abstracts the concept of bid.
//! * the [crate::fees::payment] module where the withdrawing of fees takes place.

use crate::Balance;
use frame_support::traits::{WithdrawReason, WithdrawReasons};

pub mod bid;
pub mod payment;

pub trait Fee {
    /// The associated [crate::Balance].
    fn value(&self) -> Balance;

    /// The associated [frame_support::traits::WithdrawReasosn].
    fn withdraw_reasons(&self) -> WithdrawReasons;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BaseFee;
impl Fee for BaseFee {
    fn value(&self) -> Balance {
        1
    }

    fn withdraw_reasons(&self) -> WithdrawReasons {
        WithdrawReason::TransactionPayment.into()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Tip(Balance);
impl Fee for Tip {
    fn value(&self) -> Balance {
        self.0
    }

    fn withdraw_reasons(&self) -> WithdrawReasons {
        WithdrawReason::Tip.into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn withdraw_reasons() {
        assert_eq!(
            BaseFee.withdraw_reasons(),
            WithdrawReason::TransactionPayment.into()
        );
        assert_eq!(Tip(123).withdraw_reasons(), WithdrawReason::Tip.into());
    }

    #[test]
    fn base_fee_value() {
        assert_eq!(BaseFee.value(), 1);
    }
}
