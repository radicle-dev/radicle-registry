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

use crate::{AccountId, Balance, Call};

use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::SignedExtension;
use sp_runtime::transaction_validity::{TransactionValidity, ValidTransaction};

pub mod bid;
pub mod payment;

use crate::fees::bid::Bid;
use crate::fees::payment::new_pay_fee;

/// The base fee serves as a disincentive to stop bad actors
/// from spamming the network in a DoS attack.
const BASE_FEE: Balance = 1;

use sp_runtime::transaction_validity::{InvalidTransaction, TransactionValidityError};

/// Pay the transaction fees with the given `bid`.
/// The bid is meant to cover all mandatory fees and have the remainder
/// used as a tip to increase the priority of the transaction in the network.
#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq)]
pub struct PayTxFee {
    pub bid: Balance,
}

impl SignedExtension for PayTxFee {
    const IDENTIFIER: &'static str = "PayTxFee";

    type AccountId = AccountId;
    type Call = Call;
    type AdditionalSigned = ();
    type DispatchInfo = frame_support::dispatch::DispatchInfo;
    type Pre = ();

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    fn validate(
        &self,
        author: &Self::AccountId,
        call: &Self::Call,
        _info: Self::DispatchInfo,
        _len: usize,
    ) -> TransactionValidity {
        let error = TransactionValidityError::Invalid(InvalidTransaction::Payment);
        let bid = Bid::new(self.bid).ok_or(error)?;
        match call {
            Call::Registry(registry_call) => {
                new_pay_fee(*author, bid, registry_call.clone()).map_err(|_| error)?;

                let mut valid_tx = ValidTransaction::default();
                valid_tx.priority = 123; //TODO(nuno): convert bid.tip.value() to u64
                Ok(valid_tx)
            }
            _ => Ok(ValidTransaction::default()),
        }
    }
}
