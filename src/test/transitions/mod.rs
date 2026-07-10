use crate::{
    error::Result,
    fsm::{Action, Event, TransitionParameters, UmlFsm, UmlFsmBuilder},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_internal_transitions_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("InternalTransitions");
    builder.add_transition(TransitionParameters::Enter { target: "StateA" });
    builder.add_enter_action("StateA", Action::from("EnterStateA"));
    builder.add_exit_action("StateA", Action::from("ExitStateA"));

    // Internal transition (no target — stays in state, no exit/entry)
    builder.add_transition(TransitionParameters::Internal {
        source: "StateA",
        event: Event("InternalEvent".into()),
        action: Some(Action("HandleInternalEvent".into())),
        guard: None,
    });

    // Self transition (target = source — triggers exit/entry)
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateA",
        event: Event("SelfTransitionEvent".into()),
        action: Some(Action("HandleSelfTransitionEvent".into())),
        guard: None,
    });

    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateB",
        event: Event("GoToB".into()),
        action: None,
        guard: None,
    });

    // Composite StateB
    let state_b = builder.add_state("StateB");
    builder.add_enter_action("StateB", Action::from("EnterStateB"));
    builder.add_exit_action("StateB", Action::from("ExitStateB"));

    builder.set_scope(Some(state_b));
    builder.add_transition(TransitionParameters::Enter { target: "StateBA" });

    // Internal transition on StateBA
    builder.add_transition(TransitionParameters::Internal {
        source: "StateBA",
        event: Event("InternalEvent".into()),
        action: Some(Action("HandleInternalEvent".into())),
        guard: None,
    });

    // Self transition on StateBA
    builder.add_transition(TransitionParameters::Event {
        source: "StateBA",
        target: "StateBA",
        event: Event("SelfTransitionEvent".into()),
        action: Some(Action("HandleSelfTransitionEvent".into())),
        guard: None,
    });

    builder.build()
}

fn build_guards_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("Guards");
    builder.add_transition(TransitionParameters::Enter { target: "StateA" });

    // Root level guarded transitions
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateA",
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToA".into())),
        guard: Some(Action("AGuard".into())),
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateB",
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToB".into())),
        guard: Some(Action("BGuard".into())),
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateC",
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToC".into())),
        guard: Some(Action("CGuard".into())),
    });

    // Composite StateC
    let state_c = builder.add_state("StateC");
    builder.set_scope(Some(state_c));
    builder.add_transition(TransitionParameters::Event {
        source: "StateC",
        target: "StateCA",
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToCA".into())),
        guard: Some(Action("CAGuard".into())),
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateC",
        target: "StateCB",
        event: Event("ChangeState".into()),
        action: Some(Action("ActionToCB".into())),
        guard: Some(Action("CBGuard".into())),
    });

    builder.build()
}

fn build_transitions_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("TestFsm");
    builder.add_transition(TransitionParameters::Enter { target: "StateA" });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateA",
        event: Event("SelfTransition".into()),
        action: Some(Action("HandleSelfTransition".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateB",
        event: Event("GoToB".into()),
        action: Some(Action("HandleGoToB".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateB",
        event: Event("GoToBDifferently".into()),
        action: Some(Action("HandleGoToBDifferently".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateC",
        event: Event("GoToC".into()),
        action: None,
        guard: None,
    });
    // Event list: one label, multiple events -> one transition per event
    builder.add_transition(TransitionParameters::Event {
        source: "StateB",
        target: "StateA",
        event: Event("GoToA".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateB",
        target: "StateA",
        event: Event("GoToADifferently".into()),
        action: None,
        guard: None,
    });
    builder.build()
}

fn build_direct_transitions_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("DirectTransitions");
    builder.add_transition(TransitionParameters::Enter { target: "StateA" });

    // Direct transition: no event, just action
    builder.add_transition(TransitionParameters::Direct {
        source: "StateA",
        target: "StateB",
        action: Some(Action("ActionToB".into())),
        guard: None,
    });

    // Direct transitions with guards
    builder.add_transition(TransitionParameters::Direct {
        source: "StateB",
        target: "StateC",
        action: Some(Action("ActionToC".into())),
        guard: Some(Action("CanGoToC".into())),
    });
    builder.add_transition(TransitionParameters::Direct {
        source: "StateB",
        target: "StateD",
        action: None,
        guard: Some(Action("CanGoToD".into())),
    });

    // Regular event-based transition
    builder.add_transition(TransitionParameters::Event {
        source: "StateB",
        target: "StateA",
        event: Event("GoToA".into()),
        action: None,
        guard: None,
    });

    builder.add_enter_action("StateD", Action::from("EnterStateD"));

    builder.build()
}

impl FsmTestData {
    pub fn guards() -> Self {
        let path = get_adjacent_file_path(file!(), "guards.puml");
        Self {
            name: "guards",
            content: include_str!("./guards.puml"),
            parsed: build_guards_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn internal_transitions() -> Self {
        let path = get_adjacent_file_path(file!(), "internal_transitions.puml");
        Self {
            name: "internal_transitions",
            content: include_str!("./internal_transitions.puml"),
            parsed: build_internal_transitions_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn direct_transitions() -> Self {
        let path = get_adjacent_file_path(file!(), "direct_transitions.puml");
        Self {
            name: "direct_transitions",
            content: include_str!("./direct_transitions.puml"),
            parsed: build_direct_transitions_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn transitions() -> Self {
        let path = get_adjacent_file_path(file!(), "transitions.puml");
        Self {
            name: "transitions",
            content: include_str!("./transitions.puml"),
            parsed: build_transitions_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
