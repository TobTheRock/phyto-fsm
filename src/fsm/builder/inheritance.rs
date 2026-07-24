use std::collections::HashSet;

use indextree::NodeId;
use itertools::Itertools;

use super::scoped_arena::ScopedArena;
use crate::fsm::model::{StateData, TransitionData};
use crate::fsm::types::Event;

/// Redirects each substate `Sub --> [*]` (a parented `Final`) onto its parent composite's
/// completion target, so a completed region fires `Parent --> Target` instead of terminating the
/// FSM. A top-level `[*]` (a root `Final`) is left alone. The precondition — every such parent
/// has a completion target — is checked by [`validation::substate_exits_have_completion`], so
/// parents without one simply carry no substate exits to redirect here.
///
/// [`validation::substate_exits_have_completion`]: super::validation::substate_exits_have_completion
pub fn redirect_substate_exits(arena: &mut ScopedArena<StateData>) {
    let mut rewrites = Vec::new();
    for id in arena.node_ids() {
        let Some(parent) = arena[id].parent() else {
            continue; // top-level `[*]` genuinely terminates
        };
        for (index, transition) in arena[id].get().transitions.iter().enumerate() {
            if let Some(completion) = completion_transition(transition, arena[parent].get()) {
                rewrites.push((id, index, completion));
            }
        }
    }
    for (id, index, completion) in rewrites {
        arena[id].get_mut().transitions[index] = completion;
    }
}

/// The completion transition a substate exit becomes: a `Final` firing its region's handoff onto
/// `parent`'s completion target. `Sub --> [*] : Ev` stays event-driven; an event-less
/// `Sub --> [*]` becomes an autonomous (`Direct`) completion. Returns `None` for any non-`Final`
/// transition (which is why the missing-target case is an `expect`, not a skip: only a real
/// substate exit reaches it, and `substate_exits_have_completion` validated its target exists).
fn completion_transition(
    transition: &TransitionData,
    parent: &StateData,
) -> Option<TransitionData> {
    let TransitionData::Final {
        source,
        event,
        action,
        guard,
    } = transition
    else {
        return None;
    };
    let target = parent
        .completion_target()
        .expect("substate exit implies a parent completion target");
    Some(match event {
        Some(event) => TransitionData::Event {
            source: *source,
            event: event.clone(),
            target,
            action: action.clone(),
            guard: guard.clone(),
        },
        None => TransitionData::Direct {
            source: *source,
            target,
            action: action.clone(),
            guard: guard.clone(),
        },
    })
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
