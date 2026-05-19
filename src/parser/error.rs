#[derive(PartialEq, Eq, Debug, thiserror::Error)]
pub enum ParseError {
    #[error("PlantUML grammar error: {0}")]
    Grammar(String),
    #[error("Empty input")]
    EmptyInput,
    #[error("Missing source state in transition")]
    MissingSourceState,
    #[error("Missing destination state in transition")]
    MissingDestinationState,
    #[error("Missing name in composite state")]
    MissingCompositeStateName,
    #[error("Missing state name in state description")]
    MissingStateName,
    #[error("Missing description in state description")]
    MissingDescription,
    #[error("Invalid transition description: {0}")]
    InvalidTransitionDescription(String),
    #[error("Transition must have at least an event, guard, or action")]
    EmptyTransition,
    #[error("Invalid state description: {0}")]
    InvalidStateDescription(String),
    #[error("Unrecognised state description: {0}")]
    UnrecognisedStateDescription(String),
    #[error("Expected entry or exit action")]
    MissingStateAction,
    #[error("Action name is required")]
    MissingActionName,
    #[error("Event name is required")]
    MissingEventName,
}

impl From<ParseError> for crate::error::Error {
    fn from(e: ParseError) -> Self {
        Self::Parse(e.to_string())
    }
}
