use itertools::Itertools;

use crate::error::Result;
use crate::fsm::types::{Action, Event};

use super::error::BuildError;

use super::scoped_arena::ScopedArena;
use crate::fsm::model::StateData;

pub fn injective_action_mapping(arena: &ScopedArena<StateData>) -> Result<()> {
    let action_events = arena
        .iter()
        .flat_map(|node| node.get().transitions.iter())
        .filter(|t| t.event.is_some())
        .dedup_by(|a, b| (a.event == b.event) && (a.action == b.action))
        .filter_map(|t| {
            let event = t.event.clone()?;
            t.action.as_ref().map(|action| (action.clone(), event))
        });

    action_events
        .chunk_by(|(action, _)| action.clone())
        .into_iter()
        .try_for_each(|(action, group)| {
            let items = group.collect_vec();
            if items.len() == 1 {
                Ok(())
            } else {
                let events: String = Itertools::intersperse(
                    items.into_iter().map(|(_, event)| String::from(event)),
                    ", ".to_owned(),
                )
                .collect();
                Err(BuildError::MultipleEventsPerAction { action, events }.into())
            }
        })
}

pub fn no_conflicting_transitions(arena: &ScopedArena<StateData>) -> Result<()> {
    for_each_transition_group(arena, |state_name, event, guards| {
        let has_guards = guards.len() > 1;
        let all_transitions_guarded = guards.iter().all(|g| g.is_some());
        if has_guards && !all_transitions_guarded {
            return Err(BuildError::ConflictingTransitions {
                state: state_name.to_string(),
                event: event.clone(),
            }
            .into());
        }
        Ok(())
    })
}

pub fn unique_guards_per_event(arena: &ScopedArena<StateData>) -> Result<()> {
    for_each_transition_group(arena, |_state_name, event, guards| {
        if !guards.iter().all_unique() {
            return Err(BuildError::DuplicateGuard(event.clone()).into());
        }
        Ok(())
    })
}

fn for_each_transition_group(
    arena: &ScopedArena<StateData>,
    mut validate: impl FnMut(&str, &Option<Event>, &[Option<Action>]) -> Result<()>,
) -> Result<()> {
    arena
        .iter()
        .flat_map(|node| node.get().transitions.iter())
        .map(|t| (t.source, t.event.clone(), t.guard.clone()))
        .chunk_by(|(source, event, _)| (*source, event.clone()))
        .into_iter()
        .try_for_each(|((source, event), group)| {
            let guards = group.map(|(_, _, guard)| guard).collect_vec();
            let state_name = &arena[source].get().name;
            validate(state_name, &event, &guards)
        })
}
