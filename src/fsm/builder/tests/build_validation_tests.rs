use crate::fsm::{TransitionParameters, UmlFsmBuilder};

#[test]
fn build_without_enter_state_fails() {
    let builder = UmlFsmBuilder::new("TestFSM");
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_empty_name_fails() {
    let mut builder = UmlFsmBuilder::new("  ");
    builder.add_enter_state("Start");
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_duplicate_events_per_action_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("Start");
    builder.add_transition(TransitionParameters {
        source: "Start",
        target: Some("End"),
        event: Some("EventA".into()),
        action: Some("DuplicateAction".into()),
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "Start",
        target: Some("End"),
        event: Some("EventB".into()),
        action: Some("DuplicateAction".into()),
        guard: None,
    });
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_conflicting_transitions_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("A");
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("B"),
        event: Some("EventA".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("C"),
        event: Some("EventA".into()),
        action: None,
        guard: None,
    });
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_guarded_conflicting_transitions_succeeds() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("A");
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("B"),
        event: Some("EventA".into()),
        action: None,
        guard: Some("GuardOne".into()),
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("C"),
        event: Some("EventA".into()),
        action: None,
        guard: Some("GuardTwo".into()),
    });
    let result = builder.build();
    assert!(result.is_ok());
}

#[test]
fn build_with_partially_guarded_conflicting_transitions_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("A");
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("B"),
        event: Some("EventA".into()),
        action: None,
        guard: Some("GuardOne".into()),
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("C"),
        event: Some("EventA".into()),
        action: None,
        guard: None,
    });
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_duplicate_guards_per_event_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("A");
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("B"),
        event: Some("EventA".into()),
        action: None,
        guard: Some("DuplicateGuard".into()),
    });
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("C"),
        event: Some("EventA".into()),
        action: None,
        guard: Some("DuplicateGuard".into()),
    });
    let result = builder.build();
    assert!(result.is_err());
}
