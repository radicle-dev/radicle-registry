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

//! Fee charging logic as [SignedExtension] for [PayTxFee].

use crate::fees::payment::pay;
use crate::{AccountId, Balance, Call};

use parity_scale_codec::{Decode, Encode};
use sp_runtime::traits::SignedExtension;
use sp_runtime::transaction_validity::{
    InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
};

mod payment;

/// The minimum fee required for a transaction to be considered
/// for inclusion in the blockchain.
pub const MINIMUM_FEE: Balance = 1;

/// Pay the transaction fee indicated by the author.
/// The fee should be higher or equal to [MINIMUM_FEE].
/// The higher the fee, the higher the priority of a transaction.
#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq)]
pub struct PayTxFee {
    pub fee: Balance,
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
        if self.fee < MINIMUM_FEE {
            return Err(error);
        }
        pay(*author, self.fee, &call).map_err(|_| error)?;

        let mut valid_tx = ValidTransaction::default();
        valid_tx.priority = self.fee as u64;
        Ok(valid_tx)
    }
}
