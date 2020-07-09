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

//! Access to runtime events and helpers to extract events for transactions.
use radicle_registry_core::TransactionError;
use radicle_registry_runtime::{event, DispatchError};

pub use radicle_registry_runtime::event::{transaction_index, Event, Record, Registry, *};

#[derive(thiserror::Error, Debug)]
pub enum EventExtractionError {
    #[error("ExtrinsicSuccess or ExtrinsicFailed event not found")]
    ExstrinsicStatusMissing,
    #[error("Required event is missing")]
    EventMissing,
}

/// Run `f` on all events to extract a potential output after [get_dispatch_result] is successful.
/// If `f` returns `None` for all events an [EventExtractionError::EventMissing] error is returned.
pub fn extract_registry_result<T>(
    events: &[Event],
    f: impl Fn(&event::Registry) -> Option<T>,
) -> Result<Result<T, TransactionError>, EventExtractionError> {
    let dispatch_result = get_dispatch_result(events)?;
    match dispatch_result {
        Ok(()) => {
            let output = events
                .iter()
                .find_map(|event| match event {
                    Event::registry(event) => f(event),
                    _ => None,
                })
                .ok_or_else(|| EventExtractionError::EventMissing)?;
            Ok(Ok(output))
        }
        Err(dispatch_error) => Ok(Err(dispatch_error)),
    }
}

/// Looks for `ExtrinsicSuccess` and `ExtrinsicFailed` in the events and constructs the inner
/// result accordingly. Returns an [EventExtractionError::ExstrinsicStatusMissing] error if none of
/// these events is found.
pub fn get_dispatch_result(
    events: &[Event],
) -> Result<Result<(), TransactionError>, EventExtractionError> {
    events
        .iter()
        .find_map(|event| extrinsic_result(event).map(|e| e.map_err(TransactionError::from)))
        .ok_or_else(|| EventExtractionError::ExstrinsicStatusMissing)
}

/// Extracts the extrinsic result from the event.
///
/// If the event is either `ExtrinsicSuccess` or `ExtrinsicFailed` it returns `Ok` or the
/// `Err`, respectively. If the event is neither of those it returns `None`.
fn extrinsic_result(event: &Event) -> Option<Result<(), DispatchError>> {
    match event {
        Event::system(event) => match event {
            event::System::ExtrinsicSuccess(_) => Some(Ok(())),
            event::System::ExtrinsicFailed(dispatch_error, _) => Some(Err(*dispatch_error)),
            _ => None,
        },
        _ => None,
    }
}
