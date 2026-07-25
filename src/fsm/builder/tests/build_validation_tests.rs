use crate::fsm::{TransitionParameters, UmlFsmBuilder};

#[test]
fn build_without_enter_state_fails() {
    let builder = UmlFsmBuilder::new("TestFSM");
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_multiple_enter_states_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "Start" });
    builder.add_transition(TransitionParameters::Enter {
        target: "AnotherStart",
    });
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_empty_name_fails() {
    let mut builder = UmlFsmBuilder::new("  ");
    builder.add_transition(TransitionParameters::Enter { target: "Start" });
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_duplicate_events_per_action_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "Start" });
    builder.add_transition(TransitionParameters::Event {
        source: "Start",
        target: "End",
        event: "EventA".into(),
        action: Some("DuplicateAction".into()),
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Start",
        target: "End",
        event: "EventB".into(),
        action: Some("DuplicateAction".into()),
        guard: None,
    });
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_conflicting_transitions_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "A" });
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "B",
        event: "EventA".into(),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "C",
        event: "EventA".into(),
        action: None,
        guard: None,
    });
    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn build_with_guarded_conflicting_transitions_succeeds() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "A" });
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "B",
        event: "EventA".into(),
        action: None,
        guard: Some("GuardOne".into()),
    });
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "C",
        event: "EventA".into(),
        action: None,
        guard: Some("GuardTwo".into()),
    });
    let result = builder.build();
    assert!(result.is_ok());
}

#[test]
fn build_with_guarded_and_unguarded_default_succeeds() {
    // A guarded transition plus one unguarded "else" branch is deterministic: the guard is tried
    // first, the default catches the rest.
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "A" });
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "B",
        event: "EventA".into(),
        action: None,
        guard: Some("GuardOne".into()),
    });
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "C",
        event: "EventA".into(),
        action: None,
        guard: None,
    });
    let result = builder.build();
    assert!(result.is_ok());
}

#[test]
fn build_with_substate_exit_but_no_completion_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "Working" });
    let working = builder.add_state("Working");

    builder.set_scope(Some(working));
    builder.add_transition(TransitionParameters::Enter { target: "Busy" });
    // `Busy` exits its region to `[*]`, but `Working` has no completion transition to redirect onto.
    builder.add_transition(TransitionParameters::Final {
        source: "Busy",
        event: Some("Finish".into()),
        action: None,
        guard: None,
    });
    builder.set_scope(None);

    let err = builder.build().unwrap_err();
    assert!(
        err.to_string().contains("no completion transition"),
        "expected SubstateExitWithoutCompletion, got: {err}"
    );
}

#[test]
fn build_with_duplicate_guards_per_event_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "A" });
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "B",
        event: "EventA".into(),
        action: None,
        guard: Some("DuplicateGuard".into()),
    });
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "C",
        event: "EventA".into(),
        action: None,
        guard: Some("DuplicateGuard".into()),
    });
    let result = builder.build();
    assert!(result.is_err());
}
