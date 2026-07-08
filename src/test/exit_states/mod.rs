use crate::{
    error::Result,
    fsm::{
        Action, Event, StateType, TransitionParameters, TransitionTarget, UmlFsm, UmlFsmBuilder,
    },
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_exit_states_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("ExitStates");
    builder.add_state("Active", StateType::Enter);
    builder.add_exit_action("Active", Action::from("Cleanup"));

    // Self transition
    builder.add_transition(TransitionParameters {
        source: "Active",
        target: TransitionTarget::State("Active"),
        event: Some(Event("Work".into())),
        action: None,
        guard: None,
    });

    // Exit to the `[*]` final pseudo-state: ends the FSM
    builder.add_transition(TransitionParameters {
        source: "Active",
        target: TransitionTarget::Final,
        event: Some(Event("Shutdown".into())),
        action: Some(Action("Goodbye".into())),
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
}
