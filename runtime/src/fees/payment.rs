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

use crate::{fees::Fee, AccountId, DispatchError};
use radicle_registry_core::*;

use frame_support::traits::{Currency, ExistenceRequirement};

/// Pay a given fee by withdrawing it from the `payee` account
/// and transfering it, with a small burn, to the block author.
pub fn pay_fee(fee: impl Fee, payee: &AccountId) -> Result<(), DispatchError> {
    // 1. Withdraw from payee
    let withdraw_result = <crate::Balances as Currency<_>>::withdraw(
        payee,
        fee.value(),
        fee.withdraw_reason().into(),
        ExistenceRequirement::KeepAlive,
    );
    let _negative_imbalance = match withdraw_result {
        Ok(x) => x,
        Err(_e) => return Err(RegistryError::FailedFeePayment.into()),
    };

    Ok(())
    // 2. Transfer to ??? TODO(nuno)
}

/// The burn associated with the payment of a fee.
/// When a tx fee is withdrew, it is then transfered to the block author.
/// We apply a small burn on that transfer to increase the value of our
/// currency. We will burn this percentage and then floor to go back to Balance.
const _FEE_PAYMENT_BURN: f64 = 0.01;


pub fn can_pay(bid: Balance, payee: &AccountId) -> Result<(), DispatchError> {
    // <crate::Balances as Currency<_>>::ensure_can_withdraw(
    //     payee,
    //     bid,
    //     frame_support::traits::WithdrawReason::TransactionPayment.into(),
    // )

    // fn ensure_can_withdraw(
    //     who: &AccountId,
    //     _amount: Self::Balance,
    //     reasons: WithdrawReasons,
    //     new_balance: Self::Balance
    // ) -> DispatchResult

    Ok(())
}