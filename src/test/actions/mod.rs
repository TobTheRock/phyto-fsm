use crate::{
    error::Result,
    fsm::{Action, Event, TransitionParameters, UmlFsm, UmlFsmBuilder},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_actions_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("TestFsm");
    builder.add_transition(TransitionParameters::Enter { target: "StateA" });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateB",
        event: Event("GoToB".into()),
        action: Some(Action("HandleGoToB".into())),
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateB",
        target: "StateA",
        event: Event("GoToA".into()),
        action: Some(Action("HandleGoToA".into())),
        guard: None,
    });
    builder.build()
}

fn build_enter_exit_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("EnterExitActions");

    // Root level states
    builder.add_transition(TransitionParameters::Enter { target: "StateA" });
    builder.add_enter_action("StateA", Action::from("EnterStateA"));
    builder.add_exit_action("StateA", Action::from("ExitStateA"));
    builder.add_state("StateB");

    // Root level transitions
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateA",
        event: Event::from("GoToAFromA"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateB",
        event: Event::from("GoToB"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateB",
        target: "StateA",
        event: Event::from("GoToAFromB"),
        action: None,
        guard: None,
    });

    // Composite state C
    let state_c = builder.add_state("StateC");
    builder.add_enter_action("StateC", Action::from("EnterStateC"));
    builder.add_exit_action("StateC", Action::from("ExitStateC"));

    // C's children
    builder.set_scope(Some(state_c));
    builder.add_transition(TransitionParameters::Enter { target: "StateCA" });
    builder.add_enter_action("StateCA", Action::from("EnterStateCA"));
    builder.add_exit_action("StateCA", Action::from("ExitStateCA"));
    builder.add_state("StateCB");
    builder.add_transition(TransitionParameters::Event {
        source: "StateCA",
        target: "StateCB",
        event: Event::from("GoToCB"),
        action: None,
        guard: None,
    });

    // Root level transitions involving C
    builder.set_scope(None);
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateC",
        event: Event::from("GoToC"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateCA",
        event: Event::from("GoToCAFromA"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateA",
        target: "StateCB",
        event: Event::from("GoToCBFromA"),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "StateC",
        target: "StateA",
        event: Event::from("GoToAFromC"),
        action: None,
        guard: None,
    });

    builder.build()
}

impl FsmTestData {
    pub fn actions() -> Self {
        let path = get_adjacent_file_path(file!(), "actions.puml");
        Self {
            name: "actions",
            content: include_str!("./actions.puml"),
            parsed: build_actions_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn enter_exit() -> Self {
        let path = get_adjacent_file_path(file!(), "enter_exit.puml");
        Self {
            name: "enter_exit",
            content: include_str!("./enter_exit.puml"),
            parsed: build_enter_exit_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
