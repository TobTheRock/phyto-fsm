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

/// An event-less ("completion") exit: `Done --> [*]` fires as soon as `Done` is reached,
/// tearing it down and ending the FSM without an explicit event.
fn build_completion_exit_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("CompletionExit");
    builder.add_transition(TransitionParameters::Enter { target: "Active" });
    builder.add_transition(TransitionParameters::Event {
        source: "Active",
        target: "Done",
        event: Event("Finish".into()),
        action: None,
        guard: None,
    });
    builder.add_exit_action("Done", Action::from("Cleanup"));
    builder.add_transition(TransitionParameters::Final {
        source: "Done",
        event: None,
        action: None,
        guard: None,
    });

    builder.build()
}

/// A substate exiting to `[*]` completes its region rather than ending the FSM: `Idle --> [*]`
/// inside `Working` fires `Working`'s completion transition (`Working --> Done`). The builder
/// lowers the substate `Final` onto that completion target — only the top-level `Done --> [*]`
/// terminates.
fn build_substate_exit_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("SubstateExit");
    builder.add_transition(TransitionParameters::Enter { target: "Working" });
    let working = builder.add_state("Working");

    builder.set_scope(Some(working));
    builder.add_transition(TransitionParameters::Enter { target: "Busy" });
    builder.add_transition(TransitionParameters::Event {
        source: "Busy",
        target: "Idle",
        event: Event("Pause".into()),
        action: None,
        guard: None,
    });
    // Substate exit: lowered onto Working's completion target (`Done`), keeping its `Finish` event
    builder.add_transition(TransitionParameters::Final {
        source: "Idle",
        event: Some(Event("Finish".into())),
        action: None,
        guard: None,
    });
    builder.set_scope(None);

    // Completion transition: where Working goes once its region is done
    builder.add_transition(TransitionParameters::Direct {
        source: "Working",
        target: "Done",
        action: None,
        guard: None,
    });
    // Top-level exit: ends the FSM
    builder.add_transition(TransitionParameters::Final {
        source: "Done",
        event: Some(Event("Shutdown".into())),
        action: None,
        guard: None,
    });

    builder.build()
}

/// A substate exit fans out over guarded completion transitions: `Idle --> [*] : Finish` inside
/// `Working` becomes both `Idle --Finish[LowBattery]--> Recharge / Cleanup` and
/// `Idle --Finish[Ok]--> Done`, each carrying that completion's guard and effect. The consumed
/// `Working --> ...` completions are dropped from the composite.
fn build_guarded_completion_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("GuardedCompletion");
    builder.add_transition(TransitionParameters::Enter { target: "Working" });
    let working = builder.add_state("Working");

    builder.set_scope(Some(working));
    builder.add_transition(TransitionParameters::Enter { target: "Busy" });
    builder.add_transition(TransitionParameters::Event {
        source: "Busy",
        target: "Idle",
        event: Event("Pause".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Final {
        source: "Idle",
        event: Some(Event("Finish".into())),
        action: None,
        guard: None,
    });
    builder.set_scope(None);

    // Guarded completion transitions Working fans out over
    builder.add_transition(TransitionParameters::Direct {
        source: "Working",
        target: "Recharge",
        action: Some(Action::from("Cleanup")),
        guard: Some(Action::from("LowBattery")),
    });
    builder.add_transition(TransitionParameters::Direct {
        source: "Working",
        target: "Done",
        action: None,
        guard: Some(Action::from("Ok")),
    });
    // Top-level exit: ends the FSM
    builder.add_transition(TransitionParameters::Final {
        source: "Done",
        event: Some(Event("Shutdown".into())),
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

    pub fn completion_exit() -> Self {
        let path = get_adjacent_file_path(file!(), "completion_exit.puml");
        Self {
            name: "completion_exit",
            content: include_str!("./completion_exit.puml"),
            parsed: build_completion_exit_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn substate_exit() -> Self {
        let path = get_adjacent_file_path(file!(), "substate_exit.puml");
        Self {
            name: "substate_exit",
            content: include_str!("./substate_exit.puml"),
            parsed: build_substate_exit_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }

    pub fn guarded_completion() -> Self {
        let path = get_adjacent_file_path(file!(), "guarded_completion.puml");
        Self {
            name: "guarded_completion",
            content: include_str!("./guarded_completion.puml"),
            parsed: build_guarded_completion_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
