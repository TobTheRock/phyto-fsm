use crate::fsm::types::{Action, Event};

use super::StateId;
use super::transition::{Transition, TransitionData};

#[derive(Debug, Clone)]
pub struct StateData {
    pub name: String,
    pub transitions: Vec<super::TransitionData>,
    pub enter_action: Option<Action>,
    pub exit_action: Option<Action>,
    /// Includes the inherited events from potential parents
    pub deferred_events: Vec<Event>,
}

impl StateData {
    /// Whether this state is the initial state of its scope (owns an [`Enter`] transition).
    ///
    /// [`Enter`]: TransitionData::Enter
    pub fn is_enter(&self) -> bool {
        self.transitions
            .iter()
            .any(|t| matches!(t, TransitionData::Enter { .. }))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct State<'a> {
    id: StateId,
    arena: &'a indextree::Arena<StateData>,
}

impl<'a> State<'a> {
    pub fn new(id: StateId, arena: &'a indextree::Arena<StateData>) -> Self {
        Self { id, arena }
    }

    pub fn name(&self) -> &str {
        &self.node_data().name
    }

    /// Whether this state is the initial state of its scope (owns an [`Enter`] transition).
    ///
    /// [`Enter`]: TransitionData::Enter
    pub fn is_enter(&self) -> bool {
        self.node_data().is_enter()
    }

    pub fn enter_action(&self) -> Option<&Action> {
        self.node_data().enter_action.as_ref()
    }

    pub fn exit_action(&self) -> Option<&Action> {
        self.node_data().exit_action.as_ref()
    }

    /// The state's real outgoing transitions. The `Enter` pseudo-transition (whose source is
    /// `[*]`, not this state) is excluded — query it via [`is_enter`](Self::is_enter).
    pub fn transitions(&self) -> impl Iterator<Item = Transition<'_>> {
        let arena = self.arena;
        self.node_data()
            .transitions
            .iter()
            .filter(|t| !matches!(t, TransitionData::Enter { .. }))
            .map(move |t| Transition::from(t, arena))
    }

    pub fn parent(&self) -> Option<State<'a>> {
        self.node()
            .parent()
            .map(|parent_id| State::new(parent_id, self.arena))
    }

    pub fn substates(&self) -> impl Iterator<Item = State<'a>> {
        self.id
            .children(self.arena)
            .map(move |child_id| State::new(child_id, self.arena))
    }

    /// The state entered when this state's scope becomes active: the deepest nested initial
    /// substate (each composite descends into its `[*] -->` child), or `self` if it is a leaf.
    pub fn enter_state(&self) -> State<'a> {
        match self.substates().find(|s| s.is_enter()) {
            Some(child) => child.enter_state(),
            None => *self,
        }
    }

    pub fn deferred_events(&self) -> impl Iterator<Item = &Event> {
        self.node_data().deferred_events.iter()
    }

    fn node(&self) -> &indextree::Node<StateData> {
        &self.arena[self.id]
    }

    fn node_data(&self) -> &StateData {
        self.node().get()
    }
}

impl<'a> PartialEq for State<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.name() == other.name()
            && self.is_enter() == other.is_enter()
            && self.parent() == other.parent()
    }
}
