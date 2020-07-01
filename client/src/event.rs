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

//! Access to runtime events of different runtime versions.
//!
//! This module provides facilities to deal with different versions of runtime events and extract
//! information from them.
use parity_scale_codec::{DecodeAll as _, Encode as _, Error as ScaleCodecError};
use radicle_registry_core::TransactionError;
use radicle_registry_runtime::{event as event_v2, DispatchError, Event as EventV2};
use radicle_registry_runtime_v11::{event as event_v1, Event as EventV1};

pub use event_v2::Registry;

/// Compatibility adapter for runtime [event_v2::Record].
pub enum EventRecord {
    V1(event_v1::Record),
    V2(event_v2::Record),
}

impl EventRecord {
    /// Decode a list of events from the state storage.
    ///
    /// Requires the runtime spec version that executed the block the events belong to.
    pub fn decode_vec(
        runtime_spec_version: u32,
        data: &[u8],
    ) -> Result<Vec<Self>, ScaleCodecError> {
        if runtime_spec_version <= radicle_registry_runtime_v11::VERSION.spec_version {
            let records = Vec::<event_v1::Record>::decode_all(data)?;
            Ok(records.into_iter().map(EventRecord::V1).collect())
        } else {
            let records = Vec::<event_v2::Record>::decode_all(data)?;
            Ok(records.into_iter().map(EventRecord::V2).collect())
        }
    }

    pub fn event(&self) -> Event {
        match self {
            EventRecord::V1(record) => Event::V1(record.event.clone()),
            EventRecord::V2(record) => Event::V2(record.event.clone()),
        }
    }

    /// Return the index of the transaction in the block that dispatched the event.
    ///
    /// Returns `None` if the event was not dispatched as part of a transaction.
    pub fn transaction_index(&self) -> Option<u32> {
        match self {
            EventRecord::V1(record) => event_v1::transaction_index(record),
            EventRecord::V2(record) => event_v2::transaction_index(record),
        }
    }
}

/// Compatibility adapter for runtime [EventV2].
pub enum Event {
    V1(EventV1),
    V2(EventV2),
}

impl Event {
    /// Extracts the extrinsic result from the event.
    ///
    /// If the event is either `ExtrinsicSuccess` or `ExtrinsicFailed` it returns `Ok` or the
    /// `Err`, respectively. If the event is neither of those it returns `None`.
    pub fn extrinsic_result(&self) -> Option<Result<(), DispatchError>> {
        match self {
            Event::V1(event) => match event {
                EventV1::system(event) => match event {
                    event_v1::System::ExtrinsicSuccess(_) => Some(Ok(())),
                    event_v1::System::ExtrinsicFailed(dispatch_error, _) => {
                        // The SCALE representation of `DisptachError` has not changed between the
                        // two runtime versions.
                        let dispatch_error =
                            sp_runtime::DispatchError::decode_all(&dispatch_error.encode())
                                .expect("");
                        Some(Err(dispatch_error))
                    }
                    _ => None,
                },
                _ => None,
            },
            Event::V2(event) => match event {
                EventV2::system(event) => match event {
                    event_v2::System::ExtrinsicSuccess(_) => Some(Ok(())),
                    event_v2::System::ExtrinsicFailed(dispatch_error, _) => {
                        Some(Err(*dispatch_error))
                    }
                    _ => None,
                },
                _ => None,
            },
        }
    }

    /// Returns the inner registry event if the runtime event is a registry event
    pub fn registry(&self) -> Option<event_v2::Registry> {
        match self {
            Event::V1(EventV1::registry(event)) => Some(
                // The SCALE representation of a registry event has not changed between the two
                // runtime versions.
                event_v2::Registry::decode_all(&event.encode())
                    .expect("Incompatible registry event"),
            ),
            Event::V2(EventV2::registry(event)) => Some(event.clone()),
            _ => None,
        }
    }

    /// Returns `Some(())` if the inner event is `CodeUpdate` from [event_v2::System] or the
    /// corresponding legacy version of that event.
    pub fn system_code_update(&self) -> Option<()> {
        match self {
            Event::V1(EventV1::system(event_v1::System::CodeUpdated)) => Some(()),
            Event::V2(EventV2::system(event_v2::System::CodeUpdated)) => Some(()),
            _ => None,
        }
    }
}

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
    f: impl Fn(&event_v2::Registry) -> Option<T>,
) -> Result<Result<T, TransactionError>, EventExtractionError> {
    let dispatch_result = get_dispatch_result(events)?;
    match dispatch_result {
        Ok(()) => {
            let output = events
                .iter()
                .find_map(|event| event.registry().and_then(|e| f(&e)))
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
        .find_map(|event| {
            event
                .extrinsic_result()
                .map(|e| e.map_err(TransactionError::from))
        })
        .ok_or_else(|| EventExtractionError::ExstrinsicStatusMissing)
}
