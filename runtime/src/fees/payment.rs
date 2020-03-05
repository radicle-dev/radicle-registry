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

use crate::{fees::Fee, AccountId};
use frame_support::traits::{Currency, ExistenceRequirement};

/// Pay a given fee by withdrawing it from the `payee` account
/// and transfering it, with a small burn, to the block author.
pub fn pay_fee(fee: impl Fee, payee: &AccountId) {
    // 1. Withdraw from payee
    let _negative_imbalance = <crate::Balances as Currency<_>>::withdraw(
        payee,
        fee.value(),
        fee.withdraw_reason().into(),
        ExistenceRequirement::KeepAlive,
    );
    // 2. TODO(nuno) Transfer to the block author. Will be done in a following PR.
}

/// The burn associated with the payment of a fee.
/// When a tx fee is withdrew, it is then transfered to the block author.
/// We apply a small burn on that transfer to increase the value of our
/// currency. We will burn this percentage and then floor to go back to Balance.
const _FEE_PAYMENT_BURN: f64 = 0.01;
