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

pub use radicle_registry_core::message::*;

/// Indicates that parsing the events into the appropriate message result failed.
type EventParseError = String;

/// Trait implemented for every runtime message
///
/// For every [RuntimeCall] that is exposed to the user we implement [Message] for the parameters
/// struct of the runtime message.
pub trait Message: Send + 'static {

    fn into_runtime_call(self) -> RuntimeCall;
}

impl Message for message::RegisterProject {

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::register_project(self).into()
    }
}

impl Message for message::RegisterOrg {

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::register_org(self).into()
    }
}

impl Message for message::UnregisterOrg {
    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::unregister_org(self).into()
    }
}

impl Message for message::RegisterUser {
    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::register_user(self).into()
    }
}

impl Message for message::UnregisterUser {

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::unregister_user(self).into()
    }
}

impl Message for message::CreateCheckpoint {

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::create_checkpoint(self).into()
    }
}

impl Message for message::SetCheckpoint {
    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::set_checkpoint(self).into()
    }
}

impl Message for message::Transfer {

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::transfer(self).into()
    }
}

impl Message for message::TransferFromOrg {

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
/// Because of an issue with substrate the `message` field of [TransactionError] will always be `None`
pub fn get_dispatch_result(events: &[Event]) -> Result<Result<(), TransactionError>, EventParseError> {
    find_event(events, "System", |event| match event {
        Event::system(system_event) => match system_event {
            frame_system::Event::<Runtime>::ExtrinsicSuccess(_) => Some(Ok(())),
            frame_system::Event::<Runtime>::ExtrinsicFailed(ref dispatch_error, _) => {
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
