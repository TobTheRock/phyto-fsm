use crate::{
    error::Result,
    fsm::{Action, Event, TransitionParameters, UmlFsm, UmlFsmBuilder},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_exit_states_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("ExitStates");
    builder.add_transition(TransitionParameters::Enter { target: "Active" });
    builder.add_exit_action("Active", Action::from("Cleanup"));

    // Self transition
    builder.add_transition(TransitionParameters::Event {
        source: "Active",
        target: "Active",
        event: Event("Work".into()),
        action: None,
        guard: None,
    });

    // Exit to the `[*]` final pseudo-state: ends the FSM
    builder.add_transition(TransitionParameters::Final {
        source: "Active",
        event: Some(Event("Shutdown".into())),
        action: Some(Action("Goodbye".into())),
        guard: None,
    });

    builder.build()
}

/// A composite region that terminates the FSM: the top-level `Working` composite exits to
/// `[*]`, tearing down its active substate before ending the machine.
fn build_composite_exit_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("CompositeExit");
    builder.add_transition(TransitionParameters::Enter { target: "Working" });
    let working = builder.add_state("Working");
    builder.add_exit_action("Working", Action::from("Cleanup"));

    // Exit the whole composite region to the `[*]` final pseudo-state
    builder.add_transition(TransitionParameters::Final {
        source: "Working",
        event: Some(Event("Shutdown".into())),
        action: Some(Action("Goodbye".into())),
        guard: None,
    });

    builder.set_scope(Some(working));
    builder.add_transition(TransitionParameters::Enter { target: "Busy" });
    builder.add_transition(TransitionParameters::Event {
        source: "Busy",
        target: "Idle",
        event: Event("Pause".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Idle",
        target: "Busy",
        event: Event("Resume".into()),
        action: None,
        guard: None,
    });

    builder.build()
}

impl FsmTestData {
    pub fn exit_states() -> Self {
        let path = get_adjacent_file_path(file!(), "exit_states.puml");
        Self {
            name: "exit_states",
            content: include_str!("./exit_states.puml"),
            parsed: build_exit_states_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn composite_exit() -> Self {
        let path = get_adjacent_file_path(file!(), "composite_exit.puml");
        Self {
            name: "composite_exit",
            content: include_str!("./composite_exit.puml"),
            parsed: build_composite_exit_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
