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
// along with this progra  If not, see <https://www.gnu.org/licenses/>.

//! Defines [Message] trait and implementations for all messages in `radicle_registry_core::messages`.

use radicle_registry_core::*;
use radicle_registry_runtime::{registry, Call as RuntimeCall, Event, Runtime};

/// Indicates that parsing the events into the appropriate message result failed.
type EventParseError = String;

pub fn into_runtime_call(message: Message) -> RuntimeCall {
    match message {
        Message::RegisterOrg { org_id } => registry::Call::register_org(org_id).into(),
        Message::UnregisterOrg { org_id } => registry::Call::unregister_org(org_id).into(),
        Message::TransferFromOrg { org_id, recipient, value } => registry::Call::transfer_from_org(org_id, recipient, value).into(),

        Message::RegisterUser { user_id } => registry::Call::register_user(user_id).into(),
        Message::UnregisterUser { user_id } => registry::Call::unregister_user(user_id).into(),

        Message::CreateCheckpoint { project_hash, previous_checkpoint_id } => registry::Call::create_checkpoint(project_hash, previous_checkpoint_id).into(),
        Message::SetCheckpoint { project_name,
            org_id,
            new_checkpoint_id } => registry::Call::set_checkpoint(project_name,
                org_id,
                new_checkpoint_id).into(),

        Message::RegisterProject { project_name, org_id, checkpoint_id, metadata } => registry::Call::register_project(project_name, org_id, checkpoint_id, metadata).into(),

        Message::Transfer { recipient, balance } => registry::Call::transfer(recipient, balance).into(),
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
