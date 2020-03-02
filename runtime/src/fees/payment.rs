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

use crate::AccountId;
use crate::fees::Fee;
use frame_support::traits::{Currency, ExistenceRequirement};

/// Pay a given fee by withdrawing it from the `payee` account
/// and transfering it, with a small burn, to <another account> (TODO(nuno)).
pub fn pay_fee(fee: impl Fee, payee: &AccountId) {
    // 1. Withdraw from payee
    let _negative_imbalance = <crate::Balances as Currency<_>>::withdraw(
        payee,
        fee.value(),
        fee.withdraw_reason().into(),
        ExistenceRequirement::KeepAlive
    );
    // 2. Transfer to ??? TODO(nuno)
}

/// The burn associated with the payment of a fee.
/// When a fee is withdrew, it is then transfered to another account.
/// We apply a small burn on that transfer to increase the value of
/// our currency.
const _FEE_PAYMENT_BURN: f64 = 0.001;
