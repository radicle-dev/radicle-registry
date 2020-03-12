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

use crate::registry::store;
use crate::{
    fees::{bid::Bid, Fee},
    AccountId, DispatchError, RegistryCall,
};
use radicle_registry_core::*;

use frame_support::storage::StorageMap as _;
use frame_support::traits::{Currency, ExistenceRequirement};

type NegativeImbalance = <crate::Balances as Currency<AccountId>>::NegativeImbalance;

/// Pay Fees
/// Given a tx author, their bid, and a RegistryCall they are submitting,
/// charge the tx fees to the right account, which depends on the `registry_call`.
pub fn new_pay_fee(
    author: AccountId,
    bid: Bid,
    registry_call: RegistryCall,
) -> Result<(), DispatchError> {
    let payer = decide_payer(author, registry_call);
    let withdrawn_fee = withdraw_fee(bid, &payer)?;
    let reward = burn(withdrawn_fee);
    pay_block_author(reward)?;
    Ok(())
}

pub fn withdraw_fee(fee: impl Fee, payer: &AccountId) -> Result<NegativeImbalance, DispatchError> {
    <crate::Balances as Currency<_>>::withdraw(
        payer,
        fee.value(),
        fee.withdraw_reasons(),
        ExistenceRequirement::KeepAlive,
    )
}

/// Burn a small amount from the NegativeImbalance withdrawn from tx fee payer account.
/// TODO(nuno)
fn burn(x: NegativeImbalance) -> NegativeImbalance {
    x
}

/// TODO(nuno): Pay the block author.
fn pay_block_author(_x: NegativeImbalance) -> Result<(), DispatchError> {
    Ok(())
}

/// Decide which account will pay the tx fees for running `registry_call`.
/// For some `RegistryCall`s it should be the involved org, unless
/// the `author` is not a member and thus not authorized. In such case,
/// and for all other `RegistryCall`s, the tx author will be charged.
fn decide_payer(author: AccountId, registry_call: RegistryCall) -> AccountId {
    match who_should_pay(registry_call) {
        TxFeePayer::Org(org_id) => match store::Orgs::get(org_id) {
            Some(org) => {
                if org.members.contains(&author) {
                    org.account_id
                } else {
                    author
                }
            }
            None => author,
        },
        TxFeePayer::TxAuthor => author,
    }
}

/// Check who should pay for a given `RegistryCall`, that being
/// either the tx author or an involved Org. This function does
/// not determine whether the resolved [TxFeePayer] _must_ pay,
/// given that mal intended `RegistryCall`s might be issued that
/// need to be authorized or else bad actors would be bankrupting
/// innocent accounts.
fn who_should_pay(registry_call: RegistryCall) -> TxFeePayer {
    match registry_call {
        // Transactions payed by the org
        RegistryCall::register_project(m) => TxFeePayer::Org(m.org_id),
        RegistryCall::unregister_org(m) => TxFeePayer::Org(m.org_id),
        RegistryCall::transfer_from_org(m) => TxFeePayer::Org(m.org_id),
        RegistryCall::set_checkpoint(m) => TxFeePayer::Org(m.org_id),

        // Transactions paid by the author
        RegistryCall::create_checkpoint(_) => TxFeePayer::TxAuthor,
        RegistryCall::register_org(_) => TxFeePayer::TxAuthor,
        RegistryCall::transfer(_) => TxFeePayer::TxAuthor,

        // Match arm required by the compiler.
        crate::registry::Call::__PhantomItem(_, _) => TxFeePayer::TxAuthor,
    }
}

/// The payer of a transaction fee if the transaction is authorized
enum TxFeePayer {
    /// The given org pays for the fees
    Org(OrgId),
    /// Represents that it should be the tx author paying for the tx fees.
    TxAuthor,
}

/// The burn associated with the payment of a fee.
/// When a tx fee is withdrew, it is then transfered to the block author.
/// We apply a small burn on that transfer to increase the value of our
/// currency. We will burn this percentage and then floor to go back to Balance.
const _FEE_PAYMENT_BURN: f64 = 0.01;

//TODO(nuno): Deprecate
/// Pay a given fee by withdrawing it from the `payer` account
/// and transfering it, with a small burn, to the block author.
pub fn pay_fee(fee: impl Fee, payer: &AccountId) -> Result<(), DispatchError> {
    // 1. Withdraw from payer
    let withdraw_result = <crate::Balances as Currency<_>>::withdraw(
        payer,
        fee.value(),
        fee.withdraw_reasons(),
        ExistenceRequirement::KeepAlive,
    );
    let _negative_imbalance = match withdraw_result {
        Ok(x) => x,
        Err(_e) => return Err(RegistryError::FailedFeePayment.into()),
    };

    Ok(())
    // 2. Transfer to ??? TODO(nuno)
}
