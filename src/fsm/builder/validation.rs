use itertools::Itertools;

use crate::error::Result;
use crate::fsm::types::{Action, Event};

use super::error::BuildError;

use super::scoped_arena::ScopedArena;
use crate::fsm::model::{StateData, TransitionData};

/// Exactly one top-level state must be an enter state (`[*] -->`), i.e. the FSM's initial state.
pub fn single_root_enter(arena: &ScopedArena<StateData>) -> Result<()> {
    let enter_states = arena.root_nodes().filter(|node| node.get().is_enter());
    enter_states.clone().exactly_one().map(|_| ()).map_err(|_| {
        let names: String =
            Itertools::intersperse(enter_states.map(|node| node.get().name.as_str()), ", ")
                .collect();
        BuildError::InvalidEnterStates(names).into()
    })
}

pub fn injective_action_mapping(arena: &ScopedArena<StateData>) -> Result<()> {
    let action_events = arena
        .iter()
        .flat_map(|node| node.get().transitions.iter())
        .filter(|t| t.event().is_some())
        .dedup_by(|a, b| (a.event() == b.event()) && (a.action() == b.action()))
        .filter_map(|t| {
            let event = t.event().cloned()?;
            t.action().map(|action| (action.clone(), event))
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
        .filter(|t| !matches!(t, TransitionData::Enter { .. }))
        .map(|t| (t.source(), t.event().cloned(), t.guard().cloned()))
        .chunk_by(|(source, event, _)| (*source, event.clone()))
        .into_iter()
        .try_for_each(|((source, event), group)| {
            let guards = group.map(|(_, _, guard)| guard).collect_vec();
            let state_name = &arena[source].get().name;
            validate(state_name, &event, &guards)
        })
}
