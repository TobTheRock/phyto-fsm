use itertools::Itertools;

use crate::debug::debug;
use crate::error::Result;

use self::error::BuildError;

use super::model::{StateData, StateId, TransitionData, TransitionParameters, UmlFsm};
use super::types::{Action, Event};

mod error;
mod inheritance;
mod scoped_arena;
mod validation;
use scoped_arena::ScopedArena;

#[cfg(test)]
mod tests;

impl StateData {
    fn new(name: &str) -> Self {
        StateData {
            name: name.to_string(),
            transitions: vec![],
            enter_action: None,
            exit_action: None,
            enter_state: None,
            deferred_events: vec![],
        }
    }
}

#[derive(Debug)]
pub struct UmlFsmBuilder {
    name: String,
    arena: ScopedArena<StateData>,
}

impl UmlFsmBuilder {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            arena: ScopedArena::new(),
        }
    }

    pub fn set_scope(&mut self, scope: Option<StateId>) -> Option<StateId> {
        self.arena.set_scope(scope)
    }

    pub fn add_state(&mut self, name: &str) -> StateId {
        debug!("Adding state '{}'", name);
        self.find_state_in_scope(name)
            .unwrap_or_else(|| self.create_state(name))
    }

    /// Records a transition, creating any referenced states. Returns the state the transition
    /// is stored on (its source, or the target for an `Enter` pseudo-transition).
    pub fn add_transition(&mut self, params: TransitionParameters) -> StateId {
        debug!("Adding transition {:?}", params);
        let transition = self.resolve_transition(params);
        let owner = match transition {
            TransitionData::Enter { target } => target,
            _ => transition.source(),
        };
        self.arena[owner].get_mut().transitions.push(transition);
        owner
    }

    /// Resolves endpoint names to arena ids, creating states as needed.
    fn resolve_transition(&mut self, params: TransitionParameters) -> TransitionData {
        match params {
            TransitionParameters::Event {
                source,
                event,
                target,
                action,
                guard,
            } => TransitionData::Event {
                source: self.find_or_create_state(source),
                event,
                target: self.find_or_create_state(target),
                action,
                guard,
            },
            TransitionParameters::Internal {
                source,
                event,
                action,
                guard,
            } => TransitionData::Internal {
                source: self.find_or_create_state(source),
                event,
                action,
                guard,
            },
            TransitionParameters::Direct {
                source,
                target,
                action,
                guard,
            } => TransitionData::Direct {
                source: self.find_or_create_state(source),
                target: self.find_or_create_state(target),
                action,
                guard,
            },
            TransitionParameters::Enter { target } => TransitionData::Enter {
                target: self.find_or_create_state(target),
            },
        }
    }

    pub fn add_enter_action(&mut self, state_name: &str, action: Action) {
        debug!("Adding enter action '{}' to state '{}'", action, state_name);
        if let Some(id) = self.find_descendant_state(state_name) {
            self.arena[id].get_mut().enter_action = Some(action);
        }
    }

    pub fn add_exit_action(&mut self, state_name: &str, action: Action) {
        debug!("Adding exit action '{}' to state '{}'", action, state_name);
        if let Some(id) = self.find_descendant_state(state_name) {
            self.arena[id].get_mut().exit_action = Some(action);
        }
    }

    pub fn add_deferred_event(&mut self, state_name: &str, event: Event) {
        debug!(
            "Adding deferred event '{}' to state '{}'",
            event, state_name
        );
        if let Some(id) = self.find_descendant_state(state_name) {
            self.arena[id].get_mut().deferred_events.push(event);
        }
    }

    pub fn build(mut self) -> Result<UmlFsm> {
        debug!(
            "All states: {:?}",
            self.arena
                .iter()
                .map(|node| node.get().name.as_str())
                .collect::<Vec<_>>()
        );

        validation::injective_action_mapping(&self.arena)?;
        validation::no_conflicting_transitions(&self.arena)?;
        validation::unique_guards_per_event(&self.arena)?;

        inheritance::extract_deferred_events(&mut self.arena);
        self.link_enter_states();

        let enter_state = self.find_root_enter_state()?;
        debug!("Found root enter state: {:?}", enter_state);

        let name = self.name;
        if name.trim().is_empty() {
            return Err(BuildError::EmptyName.into());
        }

        Ok(UmlFsm::new(name, enter_state, self.arena.into_inner()))
    }

    fn find_or_create_state(&mut self, name: &str) -> StateId {
        self.find_descendant_state(name)
            .unwrap_or_else(|| self.create_state(name))
    }

    fn create_state(&mut self, name: &str) -> StateId {
        debug!(
            "Creating state '{}' in scope {:?}",
            name,
            self.arena.scope()
        );
        let state_data = StateData::new(name);
        self.arena.new_node_in_scope(state_data)
    }

    fn link_enter_states(&mut self) {
        let node_ids: Vec<_> = self
            .arena
            .iter()
            .filter_map(|node| self.arena.get_node_id(node))
            .collect();

        for id in node_ids {
            let deepest_enter = self.find_deepest_enter_state(id);
            self.arena[id].get_mut().enter_state = Some(deepest_enter);
        }
    }

    fn find_root_enter_state(&self) -> Result<StateId> {
        let enter_states = self.arena.root_nodes().filter(|node| node.get().is_enter());
        let enter_state_names = || enter_states.clone().map(|node| node.get().name.as_str());

        debug!("Root enter states: {:?}", enter_state_names().collect_vec());

        let root_enter = enter_states
            .clone()
            .filter_map(|node| self.arena.get_node_id(node))
            .exactly_one()
            .map_err(|_| {
                let names: String = Itertools::intersperse(enter_state_names(), ", ").collect();
                BuildError::InvalidEnterStates(names)
            })?;

        Ok(self.find_deepest_enter_state(root_enter))
    }

    fn find_deepest_enter_state(&self, state_id: StateId) -> StateId {
        let mut current = state_id;
        while let Some(nested_enter) = self
            .arena
            .children(current)
            .find(|child| self.arena[*child].get().is_enter())
        {
            current = nested_enter;
        }
        current
    }

    fn find_state_in_scope(&self, name: &str) -> Option<StateId> {
        self.arena
            .nodes_in_scope()
            .find(|node| node.get().name == name)
            .and_then(|node| self.arena.get_node_id(node))
    }

    fn find_descendant_state(&self, name: &str) -> Option<StateId> {
        self.arena
            .descendants_from_scope()
            .find(|node| node.get().name == name)
            .and_then(|node| self.arena.get_node_id(node))
    }
}
