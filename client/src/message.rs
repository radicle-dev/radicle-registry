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

//! Defines [Message] trait and implementations for all messages in `radicle_registry_core::messages`.

pub use radicle_registry_core::message::*;
use radicle_registry_core::*;
use radicle_registry_runtime::{call, Call as RuntimeCall};

use crate::{event, event::Event};

/// Trait implemented for every runtime message
///
/// For every [RuntimeCall] that is exposed to the user we implement [Message] for the parameters
/// struct of the runtime message.
pub trait Message: Send + 'static {
    /// Parse all runtime events emitted by the message and return the appropriate message result.
    ///
    /// Returns an error if the event list is not well formed. For example if an expected event is
    /// missing.
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError>;

    fn into_runtime_call(self) -> RuntimeCall;
}

impl Message for message::RegisterProject {
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError> {
        event::extract_registry_result(&events, |event| match event {
            event::Registry::ProjectRegistered(_, _) => Some(()),
            _ => None,
        })
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::register_project(self).into()
    }
}

impl Message for message::RegisterMember {
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError> {
        event::extract_registry_result(&events, |event| match event {
            event::Registry::MemberRegistered(_, _) => Some(()),
            _ => None,
        })
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::register_member(self).into()
    }
}

impl Message for message::RegisterOrg {
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError> {
        event::extract_registry_result(&events, |event| match event {
            event::Registry::OrgRegistered(_) => Some(()),
            _ => None,
        })
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::register_org(self).into()
    }
}

impl Message for message::UnregisterOrg {
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError> {
        event::extract_registry_result(&events, |event| match event {
            event::Registry::OrgUnregistered(_) => Some(()),
            _ => None,
        })
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::unregister_org(self).into()
    }
}

impl Message for message::RegisterUser {
    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::register_user(self).into()
    }

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError> {
        event::extract_registry_result(&events, |event| match event {
            event::Registry::UserRegistered(_) => Some(()),
            _ => None,
        })
    }
}

impl Message for message::UnregisterUser {
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError> {
        event::extract_registry_result(&events, |event| match event {
            event::Registry::UserUnregistered(_) => Some(()),
            _ => None,
        })
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::unregister_user(self).into()
    }
}

impl Message for message::Transfer {
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError> {
        event::get_dispatch_result(&events)
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::transfer(self).into()
    }
}

impl Message for message::TransferFromOrg {
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError> {
        event::get_dispatch_result(&events)
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::transfer_from_org(self).into()
    }
}

impl Message for message::UpdateRuntime {
    /// The only unequivocal sign we get that a wasm update was successful is the
    /// `RawEvent::CodeUpdated` event. Anything else is considered a failed update.
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<(), TransactionError>, event::EventExtractionError> {
        let result = events
            .into_iter()
            .find_map(|event| match event {
                event::Event::system(event::System::CodeUpdated) => Some(()),
                _ => None,
            })
            .ok_or_else(|| TransactionError::from(RegistryError::FailedChainRuntimeUpdate));
        Ok(result)
    }

    fn into_runtime_call(self) -> RuntimeCall {
        let set_code_call: RuntimeCall = call::System::set_code(self.code).into();
        call::Sudo::sudo(Box::new(set_code_call)).into()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use radicle_registry_runtime::event;
    use radicle_registry_runtime::Event;

    #[test]
    fn update_runtime_event_ok() {
        let events = vec![
            Event::system(event::System::ExtrinsicSuccess(Default::default())),
            Event::system(event::System::CodeUpdated),
        ];
        let result = message::UpdateRuntime::result_from_events(events).unwrap();
        assert_eq!(result, Ok(()))
    }

    #[test]
    fn update_runtime_empty_error() {
        let events = vec![];
        let result = message::UpdateRuntime::result_from_events(events).unwrap();
        assert_eq!(
            result,
            Err(TransactionError::from(
                RegistryError::FailedChainRuntimeUpdate
            ))
        )
    }

    #[test]
    fn update_runtime_extrinsic_failed_error() {
        let events = vec![Event::system(event::System::ExtrinsicFailed(
            sp_runtime::DispatchError::BadOrigin,
            Default::default(),
        ))];
        let result = message::UpdateRuntime::result_from_events(events).unwrap();
        assert_eq!(
            result,
            Err(TransactionError::from(
                RegistryError::FailedChainRuntimeUpdate
            ))
        )
    }
}
