use crate::{
    error::Result,
    fsm::{Action, Event, TransitionParameters, UmlFsm, UmlFsmBuilder},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_deferred_events_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("DeferredEvents");

    builder.add_enter_state("StateA");
    builder.add_enter_action("StateA", Action::from("EnterStateA"));
    builder.add_state("StateB");
    builder.add_enter_action("StateB", Action::from("EnterStateB"));
    builder.add_state("StateC");
    builder.add_enter_action("StateC", Action::from("EnterStateC"));
    builder.add_state("StateD");
    builder.add_enter_action("StateD", Action::from("EnterStateD"));

    builder.add_deferred_event("StateA", Event::from("GoToA"));

    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Some(Event("GoToB".into())),
        action: None,
        guard: None,
    });

    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateC"),
        event: Some(Event("GoToC".into())),
        action: None,
        guard: None,
    });

    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateD"),
        event: Some(Event("GoToD".into())),
        action: None,
        guard: None,
    });

    builder.add_transition(TransitionParameters {
        source: "StateB",
        target: Some("StateA"),
        event: Some(Event("GoToA".into())),
        action: None,
        guard: None,
    });

    builder.add_deferred_event("StateC", Event::from("GoToA"));

    builder.add_transition(TransitionParameters {
        source: "StateC",
        target: Some("StateB"),
        event: Some(Event("GoToBFromC".into())),
        action: None,
        guard: None,
    });

    builder.add_deferred_event("StateD", Event::from("GoToA"));

    builder.add_transition(TransitionParameters {
        source: "StateD",
        target: Some("StateB"),
        event: None,
        action: None,
        guard: None,
    });

    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateF"),
        event: Some("GoToF".into()),
        action: None,
        guard: None,
    });

    // Composite StateE with substate StateF
    let state_e = builder.add_state("StateE");
    builder.add_deferred_event("StateE", Event::from("GoToA"));

    builder.set_scope(Some(state_e));
    builder.add_state("StateF");
    builder.set_scope(None);

    // StateF -> StateB on GoToBFromF
    builder.add_transition(TransitionParameters {
        source: "StateF",
        target: Some("StateB"),
        event: Some(Event("GoToBFromF".into())),
        action: None,
        guard: None,
    });

    builder.build()
}

impl FsmTestData {
    pub fn deferred_events() -> Self {
        let path = get_adjacent_file_path(file!(), "deferred.puml");
        Self {
            name: "deferred_events",
            content: include_str!("./deferred.puml"),
            parsed: build_deferred_events_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
