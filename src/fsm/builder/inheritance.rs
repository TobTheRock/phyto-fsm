use std::collections::HashSet;

use indextree::NodeId;
use itertools::Itertools;

use super::scoped_arena::ScopedArena;
use crate::fsm::model::StateData;
use crate::fsm::types::Event;

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
