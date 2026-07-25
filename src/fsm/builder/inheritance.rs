use std::collections::HashSet;

use indextree::NodeId;
use itertools::Itertools;

use super::scoped_arena::ScopedArena;
use crate::fsm::model::{StateData, TransitionData};
use crate::fsm::types::Event;

/// Redirects each substate `Sub --> [*]` (a parented `Final`) onto its parent composite's
/// completion transitions, so a completed region hands off via `Parent --> X` instead of
/// terminating the FSM. The exit *fans out* over every completion, keeping its own trigger event
/// but taking each completion's target, effect, and guard. A top-level `[*]` (a root `Final`) is
/// left alone.
pub fn redirect_substate_exits(arena: &mut ScopedArena<StateData>) {
    let mut rewrites = Vec::new();
    let mut consumed_parents = HashSet::new();
    for id in arena.node_ids() {
        let Some(parent) = arena[id].parent() else {
            continue; // top-level `[*]` genuinely terminates
        };
        let Some(redirected) = redirect_node(arena[id].get(), arena[parent].get()) else {
            continue;
        };
        rewrites.push((id, redirected));
        consumed_parents.insert(parent);
    }
    for (id, transitions) in rewrites {
        arena[id].get_mut().transitions = transitions;
    }
    for parent in consumed_parents {
        drop_realized_completions(arena[parent].get_mut());
    }
}

fn redirect_node(node: &StateData, parent: &StateData) -> Option<Vec<TransitionData>> {
    if !node.transitions.iter().any(is_substate_exit) {
        return None;
    }
    Some(
        node.transitions
            .iter()
            .flat_map(|transition| redirect_transition(transition, parent))
            .collect(),
    )
}

fn drop_realized_completions(parent: &mut StateData) {
    parent
        .transitions
        .retain(|t| !matches!(t, TransitionData::Direct { .. }));
}

fn is_substate_exit(transition: &TransitionData) -> bool {
    matches!(transition, TransitionData::Final { .. })
}

/// A substate exit fanned out to one transition per parent completion; any other transition passes
/// through unchanged.
fn redirect_transition(transition: &TransitionData, parent: &StateData) -> Vec<TransitionData> {
    let TransitionData::Final { source, event, .. } = transition else {
        return vec![transition.clone()];
    };
    let redirected: Vec<_> = parent
        .completion_transitions()
        .map(|c| c.redirect(*source, event.clone()))
        .collect();
    assert!(
        !redirected.is_empty(),
        "substate exit implies a parent completion transition"
    );
    redirected
}

pub fn extract_deferred_events(arena: &mut ScopedArena<StateData>) {
    let ids = arena.node_ids().collect_vec();
    for id in ids {
        let extracted = extract_deferred_events_for_node(arena, id);
        let node = &mut arena[id];
        node.get_mut().deferred_events = extracted;
    }
}

fn extract_deferred_events_for_node(arena: &ScopedArena<StateData>, node_id: NodeId) -> Vec<Event> {
    let all_transition_events: HashSet<_> = ancestor_transition_events(arena, node_id).collect();
    let not_overwritten = |event: &Event| !all_transition_events.contains(event);
    ancestor_deferred_events(arena, node_id)
        .filter(|&x| not_overwritten(x))
        .cloned()
        .unique()
        .collect_vec()
}

fn ancestor_transition_events(
    arena: &ScopedArena<StateData>,
    node_id: NodeId,
) -> impl Iterator<Item = &Event> {
    arena
        .ancestors(node_id)
        .flat_map(|id| arena[id].get().transitions.iter())
        .filter_map(|t| t.event())
}

fn ancestor_deferred_events(
    arena: &ScopedArena<StateData>,
    node_id: NodeId,
) -> impl Iterator<Item = &Event> {
    arena
        .ancestors(node_id)
        .flat_map(|id| arena[id].get().deferred_events.iter())
}
