//! Defines [Call] trait and implementations for all transaction parameters.

use radicle_registry_runtime::{balances, registry, Call as RuntimeCall, Event};
use sr_primitives::DispatchError;

/// Indicates that parsing the events into the approriate call result failed.
type EventParseError = String;

/// Trait implemented for every runtime call.
///
/// For every [RuntimeCall] that is exposed to the user we implement [Call] for the parameters
/// struct of the runtime call.
pub trait Call: Send + 'static {
    /// Result of executing the call in the runtime that is presented to the client user.
    type Result: Send + 'static;

    /// Parse all runtime events emitted by the call and return the appropriate call result.
    ///
    /// Returns an error if the event list is not well formed. For example if an expected event is
    /// missing.
    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError>;

    fn into_runtime_call(self) -> RuntimeCall;
}

impl Call for registry::RegisterProjectParams {
    type Result = Result<(), Option<&'static str>>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "ProjectRegistered", |event| match event {
                Event::registry(registry::Event::ProjectRegistered(_project_id, _account_id)) => {
                    Some(Ok(()))
                }
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error.message)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::register_project(self).into()
    }
}

impl Call for registry::CreateCheckpointParams {
    type Result = Result<(), Option<&'static str>>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "CheckpointCreated", |event| match event {
                Event::registry(registry::Event::CheckpointCreated(_checkpoint_id)) => Some(Ok(())),
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error.message)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::create_checkpoint(self).into()
    }
}

impl Call for registry::SetCheckpointParams {
    type Result = Result<(), Option<&'static str>>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        match dispatch_result {
            Ok(()) => find_event(&events, "CheckpointSet", |event| match event {
                Event::registry(registry::Event::CheckpointSet(_project_id, _checkpoint_id)) => {
                    Some(Ok(()))
                }
                _ => None,
            }),
            Err(dispatch_error) => Ok(Err(dispatch_error.message)),
        }
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::set_checkpoint(self).into()
    }
}

impl Call for crate::TransferParams {
    type Result = Result<(), Option<&'static str>>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        Ok(dispatch_result.map_err(|dispatch_error| dispatch_error.message))
    }

    fn into_runtime_call(self) -> RuntimeCall {
        balances::Call::transfer(self.recipient, self.balance).into()
    }
}

impl Call for registry::TransferFromProjectParams {
    type Result = Result<(), Option<&'static str>>;

    fn result_from_events(events: Vec<Event>) -> Result<Self::Result, EventParseError> {
        let dispatch_result = get_dispatch_result(&events)?;
        Ok(dispatch_result.map_err(|dispatch_error| dispatch_error.message))
    }

    fn into_runtime_call(self) -> RuntimeCall {
        registry::Call::transfer_from_project(self).into()
    }
}

/// Extract the dispatch result of an extrinsic from the extrinsic events.
///
/// Looks for the [paint_system::Event] in the list of events and returns the inner result based on
/// the event.
///
/// Returns an outer [EventParseError] if no [paint_system::Event] was present in `events`.
///
/// Because of an issue with substrate the `message` field of [DispatchError] will always be `None`
fn get_dispatch_result(events: &Vec<Event>) -> Result<Result<(), DispatchError>, EventParseError> {
    find_event(events, "System", |event| match event {
        Event::system(system_event) => match system_event {
            paint_system::Event::ExtrinsicSuccess => Some(Ok(())),
            paint_system::Event::ExtrinsicFailed(ref dispatch_error) => {
                Some(Err(dispatch_error.clone()))
            }
        },
        _ => None,
    })
}

/// Applies function to the elements of iterator and returns the first non-none result.
///
/// Returns an error if no matching element was found.
fn find_event<T>(
    events: &Vec<Event>,
    event_name: &'static str,
    f: impl Fn(&Event) -> Option<T>,
) -> Result<T, EventParseError> {
    events
        .iter()
        .find_map(f)
        .ok_or(format!("{} event is missing", event_name))
}
