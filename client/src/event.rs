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

pub use radicle_registry_runtime::event::{transaction_index, Event, Record, *};

#[derive(thiserror::Error, Debug)]
pub enum EventExtractionError {
    #[error("ExtrinsicSuccess or ExtrinsicFailed event not found")]
    ExstrinsicStatusMissing,
    #[error("Required event is missing")]
    EventMissing,
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
