use std::collections::HashSet;

use crate::fsm::types::{Action, Event};

use super::StateId;
use super::state::{State, StateData};
use super::transition::Transition;

#[derive(Clone)]
pub struct UmlFsm {
    name: String,
    enter_state: StateId,
    arena: indextree::Arena<StateData>,
}

impl UmlFsm {
    pub fn new(name: String, enter_state: StateId, arena: indextree::Arena<StateData>) -> Self {
        Self {
            name,
            enter_state,
            arena,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn enter_state(&self) -> State<'_> {
        State::new(self.enter_state, &self.arena)
    }

    pub fn states(&self) -> impl Iterator<Item = State<'_>> {
        let arena = &self.arena;
        self.arena.iter().map(move |node| {
            let id = arena.get_node_id(node).unwrap();
            State::new(id, arena)
        })
    }

    /// All real transitions across the FSM. `Enter` pseudo-transitions are excluded
    /// (see [`State::transitions`]).
    pub fn transitions(&self) -> impl Iterator<Item = Transition<'_>> {
        let arena = &self.arena;
        self.arena.iter().flat_map(move |node| {
            node.get()
                .transitions
                .iter()
                .filter(|t| !matches!(t, super::TransitionData::Enter { .. }))
                .map(|t| Transition::from(t, arena))
        })
    }
}

impl PartialEq for UmlFsm {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.enter_states_eq(other)
            && self.states_eq(other)
            && self.transitions_eq(other)
    }
}

impl Eq for UmlFsm {}

impl UmlFsm {
    fn enter_states_eq(&self, other: &Self) -> bool {
        let self_enter = self.enter_state();
        let other_enter = other.enter_state();
        self_enter.name() == other_enter.name()
            && self_enter.is_enter() == other_enter.is_enter()
    }

    fn states_eq(&self, other: &Self) -> bool {
        let self_states: HashSet<_> = self
            .states()
            .map(|s| (s.name().to_string(), s.is_enter()))
            .collect();
        let other_states: HashSet<_> = other
            .states()
            .map(|s| (s.name().to_string(), s.is_enter()))
            .collect();
        self_states == other_states
    }

    fn transitions_eq(&self, other: &Self) -> bool {
        let self_transitions: HashSet<_> = self.transitions().map(transition_key).collect();
        let other_transitions: HashSet<_> = other.transitions().map(transition_key).collect();
        self_transitions == other_transitions
    }
}

impl std::fmt::Display for UmlFsm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "UmlFsm {{")?;
        writeln!(f, "  name: {}", self.name)?;
        writeln!(f, "  states:")?;
        self.fmt_state_tree(f, self.enter_state(), 2)?;
        writeln!(f, "  transitions:")?;
        let mut transitions: Vec<_> = self.transitions().collect();
        transitions.sort();
        for t in transitions {
            writeln!(f, "    {}", t)?;
        }
        write!(f, "}}")
    }
}

impl std::fmt::Debug for UmlFsm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

impl UmlFsm {
    fn fmt_state_tree(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        state: State<'_>,
        indent: usize,
    ) -> std::fmt::Result {
        let prefix = " ".repeat(indent * 2);
        let type_marker = if state.is_enter() { "[*] " } else { "" };
        let enter = state
            .enter_action()
            .map(|a| format!(" > {}", a.0))
            .unwrap_or_default();
        let exit = state
            .exit_action()
            .map(|a| format!(" < {}", a.0))
            .unwrap_or_default();
        writeln!(
            f,
            "{}{}{}{}{}",
            prefix,
            type_marker,
            state.name(),
            enter,
            exit
        )?;
        for substate in state.substates() {
            self.fmt_state_tree(f, substate, indent + 1)?;
        }
        Ok(())
    }
}

fn transition_key(
    t: Transition,
) -> (
    Option<String>,
    Option<Event>,
    Option<Action>,
    Option<Action>,
) {
    (
        t.destination().map(|d| d.name().to_string()),
        t.event().cloned(),
        t.action().cloned(),
        t.guard().cloned(),
    )
}
