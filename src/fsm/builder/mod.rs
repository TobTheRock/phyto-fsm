use itertools::Itertools;

use crate::debug::debug;
use crate::error::Result;

use self::error::BuildError;

use super::model::{StateData, StateId, TransitionData, TransitionParameters, UmlFsm};
use super::types::{Action, Event, StateType};

mod error;
mod inheritance;
mod scoped_arena;
mod validation;
use scoped_arena::ScopedArena;

#[cfg(test)]
mod tests;

impl StateData {
    fn new(name: &str, state_type: StateType) -> Self {
        StateData {
            name: name.to_string(),
            state_type,
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

    pub fn add_state(&mut self, name: &str, state_type: StateType) -> StateId {
        debug!("Adding state '{}' of type {:?}", name, state_type);

        if let Some(id) = self.find_state_in_scope(name) {
            self.update_non_simple_state_type(id, state_type, name);
            return id;
        }

        self.create_state(name, state_type)
    }

    pub fn add_transition(&mut self, params: TransitionParameters) {
        let TransitionParameters {
            source,
            target,
            event,
            action,
            guard,
        } = params;

        debug!(
            "Adding transition from {} -> {:?}: {:?} [{:?}] / {:?}",
            source, target, event, guard, action
        );

        let from_id = self.find_or_create_state(source);
        let to_id = target.map(|t| self.find_or_create_state(t));

        let transition = TransitionData {
            source: from_id,
            target: to_id,
            event,
            action,
            guard,
        };

        self.arena[from_id].get_mut().transitions.push(transition);
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
            .unwrap_or_else(|| self.create_state(name, StateType::Simple))
    }

    fn create_state(&mut self, name: &str, state_type: StateType) -> StateId {
        debug!(
            "Creating state '{}' in scope {:?}",
            name,
            self.arena.scope()
        );
        let state_data = StateData::new(name, state_type);
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
        let enter_states = self
            .arena
            .root_nodes()
            .filter(|node| node.get().state_type == StateType::Enter);
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
            .find(|child| self.arena[*child].get().state_type == StateType::Enter)
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

    fn update_non_simple_state_type(&mut self, id: StateId, state_type: StateType, name: &str) {
        let current_type = self.arena[id].get().state_type;
        if state_type != StateType::Simple && current_type == StateType::Simple {
            debug!(
                "Updating Type of state '{}' from {:?} to {:?}",
                name, current_type, state_type
            );
            self.arena[id].get_mut().state_type = state_type;
        } else if state_type != StateType::Simple && current_type != state_type {
            debug!(
                "State '{}' already has type {:?}, ignoring {:?}",
                name, current_type, state_type
            );
        }
    }
}
