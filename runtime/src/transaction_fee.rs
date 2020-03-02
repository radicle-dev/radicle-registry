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

use crate::{AccountId, Balance};

use frame_support::traits::{Currency, ExistenceRequirement, WithdrawReason};


/// This module models our Transaction Fee setup.
/// TODO(nuno): enrich these module docs.

pub fn charge_fee(fee: TransactionFee, payee: &AccountId) {
    let result = <crate::Balances as Currency<_>>::withdraw(
        payee,
        fee.balance(),
        fee.to_withdraw_reason().into(),
        ExistenceRequirement::KeepAlive
    );
}

/// TransactionFee
pub enum TransactionFee {
    /// BaseFee is a fee paid is essentially an entry fee
    /// for the transaction into the network.
    BaseFee,

    /// Tip is a an amount indicated by the transaction author
    /// used to gain their transaction priority.
    Tip(Balance)
}

impl TransactionFee {
    fn to_withdraw_reason(&self) -> WithdrawReason {
        match self {
            TransactionFee::BaseFee => WithdrawReason::TransactionPayment,
            TransactionFee::Tip(_) => WithdrawReason::Tip,
        }
    }

    pub fn balance(&self) -> Balance {
        match self {
            TransactionFee::BaseFee => 1,
            TransactionFee::Tip(tip) => *tip
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use rand::Rng;


    #[test]
    fn withdraw_reason() {
        assert_eq!(TransactionFee::BaseFee.to_withdraw_reason(), WithdrawReason::TransactionPayment);
        assert_eq!(TransactionFee::Tip(123).to_withdraw_reason(), WithdrawReason::Tip);
    }

    #[test]
    fn base_fee_balance() {
        assert_eq!(TransactionFee::BaseFee.balance(), 1);
    }

    #[test]
    fn tip_balance() {
        for _ in 0 .. 50 {
            let random_tip: Balance = rand::thread_rng().gen();
            assert_eq!(TransactionFee::Tip(random_tip).balance(), random_tip);
        }
    }
}