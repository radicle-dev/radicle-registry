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

//! Defines [InherentData] for the registry module and implement [ProvideInherent].

use frame_support::traits::GetCallName as _;
use parity_scale_codec::{Decode, Encode};
use sp_inherents::{InherentIdentifier, IsFatalError, ProvideInherent};
use sp_runtime::RuntimeString;

use radicle_registry_core::AccountId;

use super::{Call, Module, Trait};
use crate::Hash;

const INHERENT_IDENTIFIER: InherentIdentifier = *b"registry";

/// Structured inherent data for the registry
#[derive(Encode, Decode)]
pub struct InherentData {
    pub block_author: AccountId,
}

#[cfg(feature = "std")]
impl sp_inherents::ProvideInherentData for InherentData {
    fn inherent_identifier(&self) -> &'static InherentIdentifier {
        &INHERENT_IDENTIFIER
    }

    fn provide_inherent_data(
        &self,
        inherent_data: &mut sp_inherents::InherentData,
    ) -> Result<(), sp_inherents::Error> {
        inherent_data.put_data(INHERENT_IDENTIFIER, &self)
    }

    fn error_to_string(&self, mut error: &[u8]) -> Option<String> {
        let inherent_error = CheckInherentError::decode(&mut error).ok()?;
        Some(format!("{}", inherent_error))
    }
}

/// Error returned for the [ProvideInherent] implementation of [Module].
#[derive(Encode)]
#[cfg_attr(feature = "std", derive(Decode))]
pub enum CheckInherentError {
    /// The call is forbidden for an inherent. `name` is the name of the call as returned by
    /// [frame_support::traits::GetCallName::get_call_name].
    ForbiddenCall { name: RuntimeString },
}

impl IsFatalError for CheckInherentError {
    fn is_fatal_error(&self) -> bool {
        true
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for CheckInherentError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        match self {
            CheckInherentError::ForbiddenCall { name } => {
                write!(f, "Call {} is forbidden for inherents", name)
            }
        }
    }
}

impl<T: Trait> ProvideInherent for Module<T>
where
    T: frame_system::Trait<
        AccountId = AccountId,
        Origin = crate::Origin,
        Hash = Hash,
        OnNewAccount = (),
    >,
    <T as frame_system::Trait>::Event: From<frame_system::RawEvent<AccountId>>,
    <T as frame_system::Trait>::OnKilledAccount:
        frame_support::traits::OnKilledAccount<T::AccountId>,
{
    type Call = Call<T>;
    type Error = CheckInherentError;
    const INHERENT_IDENTIFIER: InherentIdentifier = INHERENT_IDENTIFIER;

    fn create_inherent(raw_data: &sp_inherents::InherentData) -> Option<Self::Call> {
        let data = raw_data
            .get_data::<InherentData>(&INHERENT_IDENTIFIER)
            .expect("Failed to decode registry InherentData")
            .expect("InherentData for registry is missing");

        Some(Call::set_block_author(data.block_author))
    }

    fn check_inherent(
        call: &Self::Call,
        _data: &sp_inherents::InherentData,
    ) -> Result<(), Self::Error> {
        match call {
            Call::set_block_author(_) => Ok(()),
            _ => Err(CheckInherentError::ForbiddenCall {
                name: RuntimeString::from(call.get_call_name()),
            }),
        }
    }
}
