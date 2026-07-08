use itertools::Itertools;

use crate::fsm::{Action, Event, UmlFsm};

pub fn events(fsm: &UmlFsm) -> impl Iterator<Item = &Event> {
    fsm.transitions().filter_map(|t| t.event).unique()
}

pub fn actions(fsm: &UmlFsm) -> impl Iterator<Item = (&Action, &Event)> {
    fsm.transitions()
        .filter_map(|t| {
            let event = t.event?;
            t.action.map(|action| (action, event))
        })
        .unique()
}

pub fn guards(fsm: &UmlFsm) -> impl Iterator<Item = (&Action, &Event)> {
    fsm.transitions()
        .filter_map(|t| {
            let event = t.event?;
            t.guard.map(|guard| (guard, event))
        })
        .unique()
}

pub fn direct_transition_actions(fsm: &UmlFsm) -> impl Iterator<Item = &Action> {
    fsm.transitions()
        .filter(|t| t.event.is_none())
        .filter_map(|t| t.action)
        .unique()
}

pub fn direct_transition_guards(fsm: &UmlFsm) -> impl Iterator<Item = &Action> {
    fsm.transitions()
        .filter(|t| t.event.is_none())
        .filter_map(|t| t.guard)
        .unique()
}

#[derive(Debug, PartialEq, Eq)]
pub struct StateAction {
    pub action: Action,
    pub states: Vec<String>,
}

pub fn enter_actions(fsm: &UmlFsm) -> impl Iterator<Item = StateAction> + '_ {
    group_state_actions(fsm, |s| s.enter_action().cloned())
}

pub fn exit_actions(fsm: &UmlFsm) -> impl Iterator<Item = StateAction> + '_ {
    group_state_actions(fsm, |s| s.exit_action().cloned())
}

/// Groups enter/exit actions by action, collecting each owning state's qualified name. The
/// order follows the FSM's state order, keeping codegen output deterministic.
fn group_state_actions(
    fsm: &UmlFsm,
    select: impl Fn(&crate::fsm::State) -> Option<Action>,
) -> std::vec::IntoIter<StateAction> {
    let mut groups: Vec<StateAction> = Vec::new();
    for state in fsm.states() {
        let Some(action) = select(&state) else {
            continue;
        };
        let name = state.qualified_name("::");
        match groups.iter_mut().find(|grouped| grouped.action == action) {
            Some(grouped) => grouped.states.push(name),
            None => groups.push(StateAction {
                action,
                states: vec![name],
            }),
        }
    }
    groups.into_iter()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fsm::{StateType, TransitionParameters, TransitionTarget, UmlFsmBuilder};

    #[test]
    fn direct_transitions_not_in_events() {
        let mut builder = UmlFsmBuilder::new("TestFSM");
        builder.add_state("A", StateType::Enter);
        builder.add_transition(TransitionParameters {
            source: "A",
            target: TransitionTarget::State("B"),
            event: None,
            action: Some("DoSomething".into()),
            guard: None,
        });
        builder.add_transition(TransitionParameters {
            source: "B",
            target: TransitionTarget::State("A"),
            event: Some("GoBack".into()),
            action: None,
            guard: None,
        });
        let fsm = builder.build().unwrap();

        let evts: Vec<_> = events(&fsm).collect();
        assert_eq!(evts.len(), 1);
        assert_eq!(evts[0], &Event::from("GoBack"));
    }

    #[test]
    fn direct_transition_actions_separate_from_event_actions() {
        let mut builder = UmlFsmBuilder::new("TestFSM");
        builder.add_state("A", StateType::Enter);
        builder.add_transition(TransitionParameters {
            source: "A",
            target: TransitionTarget::State("B"),
            event: None,
            action: Some("DirectAction".into()),
            guard: None,
        });
        builder.add_transition(TransitionParameters {
            source: "B",
            target: TransitionTarget::State("A"),
            event: Some("GoBack".into()),
            action: Some("EventAction".into()),
            guard: None,
        });
        let fsm = builder.build().unwrap();

        let act: Vec<_> = actions(&fsm).collect();
        assert_eq!(act.len(), 1);
        assert_eq!(act[0].0, &Action::from("EventAction"));

        let direct: Vec<_> = direct_transition_actions(&fsm).collect();
        assert_eq!(direct.len(), 1);
        assert_eq!(direct[0], &Action::from("DirectAction"));
    }

    #[test]
    fn enter_actions_group_states_sharing_one_action() {
        let mut builder = UmlFsmBuilder::new("TestFSM");
        builder.add_state("A", StateType::Enter);
        builder.add_enter_action("A", "OnEnter".into());
        builder.add_state("B", StateType::Simple);
        builder.add_enter_action("B", "OnEnter".into());
        builder.add_state("C", StateType::Simple);
        builder.add_enter_action("C", "OnlyC".into());
        builder.add_transition(TransitionParameters {
            source: "A",
            target: TransitionTarget::State("B"),
            event: Some("Go".into()),
            action: None,
            guard: None,
        });
        builder.add_transition(TransitionParameters {
            source: "B",
            target: TransitionTarget::State("C"),
            event: Some("Go".into()),
            action: None,
            guard: None,
        });
        let fsm = builder.build().unwrap();

        let entered: Vec<_> = enter_actions(&fsm).collect();
        assert_eq!(
            entered,
            vec![
                StateAction {
                    action: Action::from("OnEnter"),
                    states: vec!["A".to_string(), "B".to_string()],
                },
                StateAction {
                    action: Action::from("OnlyC"),
                    states: vec!["C".to_string()],
                },
            ]
        );
    }

    #[test]
    fn direct_transition_guards_separate_from_event_guards() {
        let mut builder = UmlFsmBuilder::new("TestFSM");
        builder.add_state("A", StateType::Enter);
        builder.add_transition(TransitionParameters {
            source: "A",
            target: TransitionTarget::State("B"),
            event: None,
            action: None,
            guard: Some("DirectGuard".into()),
        });
        builder.add_transition(TransitionParameters {
            source: "A",
            target: TransitionTarget::State("C"),
            event: Some("GoToC".into()),
            action: None,
            guard: Some("EventGuard".into()),
        });
        let fsm = builder.build().unwrap();

        let g: Vec<_> = guards(&fsm).collect();
        assert_eq!(g.len(), 1);
        assert_eq!(g[0].0, &Action::from("EventGuard"));

        let direct: Vec<_> = direct_transition_guards(&fsm).collect();
        assert_eq!(direct.len(), 1);
        assert_eq!(direct[0], &Action::from("DirectGuard"));
    }
}
