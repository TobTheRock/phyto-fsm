use crate::fsm::{Action, Event};

#[derive(PartialEq, Eq, Debug, thiserror::Error)]
pub enum BuildError {
    #[error("FSM name cannot be empty")]
    EmptyName,
    #[error("FSM must have exactly one enter state, found {0}")]
    InvalidEnterStates(String),
    #[error("Action {action} is associated with multiple events: {events}")]
    MultipleEventsPerAction { action: Action, events: String },
    #[error("State '{state}' has multiple transitions for event {event:?}")]
    ConflictingTransitions { state: String, event: Option<Event> },
    #[error("Duplicate guard for event {0:?}")]
    DuplicateGuard(Option<Event>),
}

impl From<BuildError> for crate::error::Error {
    fn from(e: BuildError) -> Self {
        Self::Build(e.to_string())
    }
}
