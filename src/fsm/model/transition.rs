use crate::fsm::types::{Action, Event};

use super::StateId;
use super::state::{State, StateData};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionParameters<'a> {
    pub source: &'a str,
    pub target: TransitionTarget<'a>,
    /// No event indicates a direct transition
    pub event: Option<Event>,
    pub action: Option<Action>,
    pub guard: Option<Action>,
}

/// Where a transition leads, as named in the source syntax.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TransitionTarget<'a> {
    /// A named target state.
    State(&'a str),
    /// No target: an in-state (internal) transition.
    Internal,
    /// The `[*]` final pseudo-state that ends the enclosing region.
    Final,
}

/// Where a transition leads, at the id/storage layer.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TargetData {
    /// A named target state.
    State(StateId),
    /// No target: an in-state (internal) transition.
    Internal,
    /// The `[*]` final pseudo-state that ends the enclosing region.
    Final,
}

/// Where a transition leads, resolved against the arena (view layer).
#[derive(Debug, Clone)]
pub enum Target<'a> {
    State(State<'a>),
    Internal,
    Final,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TransitionData {
    pub source: StateId,
    pub target: TargetData,
    pub event: Option<Event>,
    pub action: Option<Action>,
    pub guard: Option<Action>,
}

#[derive(Debug, Clone)]
pub struct Transition<'a> {
    pub source: State<'a>,
    pub target: Target<'a>,
    pub event: Option<&'a Event>,
    pub action: Option<&'a Action>,
    pub guard: Option<&'a Action>,
}

impl<'a> Transition<'a> {
    pub fn from(
        data: &'a TransitionData,
        arena: &'a indextree::Arena<StateData>,
    ) -> Transition<'a> {
        let target = match &data.target {
            TargetData::State(id) => Target::State(State::new(*id, arena)),
            TargetData::Internal => Target::Internal,
            TargetData::Final => Target::Final,
        };
        Transition {
            source: State::new(data.source, arena),
            target,
            event: data.event.as_ref(),
            action: data.action.as_ref(),
            guard: data.guard.as_ref(),
        }
    }
}

impl<'a> Transition<'a> {
    /// The named target state, if any (`None` for internal and final transitions).
    #[cfg(test)]
    pub fn target_state(&self) -> Option<&State<'a>> {
        match &self.target {
            Target::State(state) => Some(state),
            Target::Internal | Target::Final => None,
        }
    }
}

impl std::fmt::Display for Transition<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let event_name = self.event.map(|e| e.0.as_str()).unwrap_or("(direct)");
        let guard = self
            .guard
            .map(|g| format!(" [{}]", g.0))
            .unwrap_or_default();
        let action = self
            .action
            .map(|a| format!(" / {}", a.0))
            .unwrap_or_default();
        let dest = match &self.target {
            Target::State(d) => d.name(),
            Target::Internal => "(internal)",
            Target::Final => "[*]",
        };
        write!(
            f,
            "{} --[{}{}{}]--> {}",
            self.source.name(),
            event_name,
            guard,
            action,
            dest
        )
    }
}

impl PartialEq for Transition<'_> {
    fn eq(&self, other: &Self) -> bool {
        self.source.name() == other.source.name() && self.event == other.event
    }
}

impl Eq for Transition<'_> {}

impl PartialOrd for Transition<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Transition<'_> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.source.name().cmp(other.source.name()).then_with(|| {
            let self_event = self.event.map(|e| e.0.as_str()).unwrap_or("");
            let other_event = other.event.map(|e| e.0.as_str()).unwrap_or("");
            self_event.cmp(other_event)
        })
    }
}
