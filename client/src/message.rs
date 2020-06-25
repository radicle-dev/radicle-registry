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

use radicle_registry_core::*;
use radicle_registry_runtime::{call, event, Call as RuntimeCall, Event};

pub use radicle_registry_core::message::*;

/// Indicates that parsing the events into the appropriate message result failed.
type EventParseError = String;

/// Trait implemented for every runtime message
///
/// For every [RuntimeCall] that is exposed to the user we implement [Message] for the parameters
/// struct of the runtime message.
pub trait Message: Send + 'static {
    /// Output of a successfully applied message.
    ///
    /// This value is extracted from the events that are dispatched when the message is executed in
    /// a block.
    type Output: Send + 'static;

    /// Parse all runtime events emitted by the message and return the appropriate message result.
    ///
    /// Returns an error if the event list is not well formed. For example if an expected event is
    /// missing.
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError>;

    fn into_runtime_call(self) -> RuntimeCall;
}

impl Message for message::RegisterProject {
    type Output = ();

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "ProjectRegistered", |event| match event {
                Event::registry(event::Registry::ProjectRegistered(_, _)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::register_project(self).into()
    }
}

impl Message for message::RegisterMember {
    type Output = ();

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "MemberRegistered", |event| match event {
                Event::registry(event::Registry::MemberRegistered(_, _)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::register_member(self).into()
    }
}

impl Message for message::RegisterOrg {
    type Output = ();

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "OrgRegistered", |event| match event {
                Event::registry(event::Registry::OrgRegistered(_)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::register_org(self).into()
    }
}

impl Message for message::UnregisterOrg {
    type Output = ();

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "OrgUnregistered", |event| match event {
                Event::registry(event::Registry::OrgUnregistered(_)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::unregister_org(self).into()
    }
}

impl Message for message::RegisterUser {
    type Output = ();

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::register_user(self).into()
    }

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;

        match dispatch_result {
            Ok(()) => find_event(&events, "UserRegistered", |event| match event {
                Event::registry(event::Registry::UserRegistered(_)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }
}

impl Message for message::UnregisterUser {
    type Output = ();

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "UserUnregistered", |event| match event {
                Event::registry(event::Registry::UserUnregistered(_)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::unregister_user(self).into()
    }
}

impl Message for message::CreateCheckpoint {
    type Output = CheckpointId;

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "CheckpointCreated", |event| match event {
                Event::registry(event::Registry::CheckpointCreated(checkpoint_id)) => {
                    Some(Ok(*checkpoint_id))
                }
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::create_checkpoint(self).into()
    }
}

impl Message for message::SetCheckpoint {
    type Output = ();

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "CheckpointSet", |event| match event {
                Event::registry(event::Registry::CheckpointSet(
                    _project_name,
                    _org_id,
                    _checkpoint_id,
                )) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::set_checkpoint(self).into()
    }
}

impl Message for message::Transfer {
    type Output = ();

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        get_dispatch_result(&events)
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::transfer(self).into()
    }
}

impl Message for message::TransferFromOrg {
    type Output = ();

    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        get_dispatch_result(&events)
    }

    fn into_runtime_call(self) -> RuntimeCall {
        call::Registry::transfer_from_org(self).into()
    }
}

impl Message for message::UpdateRuntime {
    type Output = ();

    /// The only unequivocal sign we get that a wasm update was successful is the
    /// `RawEvent::CodeUpdated` event. Anything else is considered a failed update.
    fn result_from_events(
        events: Vec<Event>,
    ) -> Result<Result<Self::Output, TransactionError>, EventParseError> {
        let result = events
            .into_iter()
            .find_map(|event| match event {
                Event::system(event::System::CodeUpdated) => Some(()),
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

/// Extract the dispatch result of an extrinsic from the extrinsic events.
///
/// Looks for the [event::System] in the list of events and returns the inner result based on
/// the event.
///
/// Returns an outer [EventParseError] if no [event::System] was present in `events`.
///
/// Because of an issue with substrate the `message` field of [TransactionError] will always be `None`
fn get_dispatch_result(events: &[Event]) -> Result<Result<(), TransactionError>, EventParseError> {
    find_event(events, "System", |event| match event {
        Event::system(system_event) => match system_event {
            event::System::ExtrinsicSuccess(_) => Some(Ok(())),
            event::System::ExtrinsicFailed(ref dispatch_error, _) => {
                Some(Err((*dispatch_error).into()))
            }
            _ => None,
        },
        _ => None,
    })
}

/// Applies function to the elements of iterator and returns the first non-none result.
///
/// Returns an error if no matching element was found.
fn find_event<T>(
    events: &[Event],
    event_name: &'static str,
    f: impl Fn(&Event) -> Option<T>,
) -> Result<T, EventParseError> {
    events
        .iter()
        .find_map(f)
        .ok_or_else(|| format!("{} event is missing", event_name))
}

#[cfg(test)]
mod test {
    use super::*;

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
