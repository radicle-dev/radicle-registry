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
use radicle_registry_runtime::{registry, Call as RuntimeCall, Event, Runtime};
use sp_runtime::DispatchError;

pub use radicle_registry_core::message::*;

/// Indicates that parsing the events into the approriate message result failed.
type EventParseError = String;

/// Trait implemented for every runtime message
///
/// For every [RuntimeCall] that is exposed to the user we implement [Message] for the parameters
/// struct of the runtime message.
pub trait Message: Send + 'static {
    /// Result of executing the message in the runtime that is presented to the client user.
    type Result: Send + 'static;

    /// Parse all runtime events emitted by the message and return the appropriate message result.
    ///
    /// Returns an error if the event list is not well formed. For example if an expected event is
    /// missing.
    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError>;

    fn into_runtime_call(self) -> RuntimeCall;
}

impl Message for message::RegisterProject {
    type Result = Result<(), DispatchError>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "ProjectRegistered", |event| match event {
                Event::registry(registry::Event::ProjectRegistered(_, _)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::register_project(self).into()
    }
}

impl Message for message::RegisterOrg {
    type Result = Result<(), DispatchError>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "OrgRegistered", |event| match event {
                Event::registry(registry::Event::OrgRegistered(_)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::register_org(self).into()
    }
}

impl Message for message::UnregisterOrg {
    type Result = Result<(), DispatchError>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "OrgUnregistered", |event| match event {
                Event::registry(registry::Event::OrgUnregistered(_)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::unregister_org(self).into()
    }
}

impl Message for message::RegisterUser {
    type Result = Result<(), DispatchError>;

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::register_user(self).into()
    }

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;

        match dispatch_result {
            Ok(()) => find_event(&events, "UserRegistered", |event| match event {
                Event::registry(registry::Event::UserRegistered(_)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }
}

impl Message for message::UnregisterUser {
    type Result = Result<(), DispatchError>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "UserUnregistered", |event| match event {
                Event::registry(registry::Event::UserUnregistered(_)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::unregister_user(self).into()
    }
}

impl Message for message::CreateCheckpoint {
    type Result = Result<CheckpointId, DispatchError>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "CheckpointCreated", |event| match event {
                Event::registry(registry::Event::CheckpointCreated(checkpoint_id)) => {
                    Some(Ok(*checkpoint_id))
                }
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::create_checkpoint(self).into()
    }
}

impl Message for message::SetCheckpoint {
    type Result = Result<(), DispatchError>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "CheckpointSet", |event| match event {
                Event::registry(registry::Event::CheckpointSet(
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
        registry::Call::set_checkpoint(self).into()
    }
}

impl Message for message::Transfer {
    type Result = Result<(), DispatchError>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        get_dispatch_result(&events)
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::transfer(self).into()
    }
}

impl Message for message::TransferFromOrg {
    type Result = Result<(), DispatchError>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        get_dispatch_result(&events)
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::transfer_from_org(self).into()
    }
}

/// Extract the dispatch result of an extrinsic from the extrinsic events.
///
/// Looks for the [frame_system::Event] in the list of events and returns the inner result based on
/// the event.
///
/// Returns an outer [EventParseError] if no [frame_system::Event] was present in `events`.
///
/// Because of an issue with substrate the `message` field of [DispatchError] will always be `None`
fn get_dispatch_result(events: &[Event]) -> Result<Result<(), DispatchError>, EventParseError> {
    find_event(events, "System", |event| match event {
        Event::system(system_event) => match system_event {
            frame_system::Event::<Runtime>::ExtrinsicSuccess(_) => Some(Ok(())),
            frame_system::Event::<Runtime>::ExtrinsicFailed(ref dispatch_error, _) => {
                Some(Err(*dispatch_error))
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
