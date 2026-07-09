use crate::fsm::types::{Action, Event, StateType};

use super::StateId;
use super::transition::Transition;

#[derive(Debug, Clone)]
pub struct StateData {
    pub name: String,
    pub state_type: StateType,
    pub transitions: Vec<super::TransitionData>,
    pub enter_action: Option<Action>,
    pub exit_action: Option<Action>,
    pub enter_state: Option<StateId>,
    /// Includes the inherited events from potential parents
    pub deferred_events: Vec<Event>,
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

    pub fn state_type(&self) -> StateType {
        self.node_data().state_type
    }

    pub fn enter_action(&self) -> Option<&Action> {
        self.node_data().enter_action.as_ref()
    }

    pub fn exit_action(&self) -> Option<&Action> {
        self.node_data().exit_action.as_ref()
    }

    pub fn transitions(&self) -> impl Iterator<Item = Transition<'_>> {
        let arena = self.arena;
        self.node_data()
            .transitions
            .iter()
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

    pub fn enter_state(&self) -> State<'a> {
        let data = self.node_data();
        if let Some(enter_id) = data.enter_state {
            State::new(enter_id, self.arena)
        } else {
            State::new(self.id, self.arena)
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
            && self.state_type() == other.state_type()
            && self.parent() == other.parent()
    }
}
