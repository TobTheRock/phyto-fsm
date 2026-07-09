use crate::{
    error::Result,
    fsm::{Event, TransitionParameters, UmlFsm, UmlFsmBuilder},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_internal_names_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("InternalNames");
    builder.add_enter_state("StateA");
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Some(Event("TriggerEvent".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Some(Event("ChangeState".into())),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Some(Event("Start".into())),
        action: None,
        guard: None,
    });
    builder.build()
}

impl FsmTestData {
    pub fn misc() -> Self {
        let path = get_adjacent_file_path(file!(), "internal_names.puml");
        Self {
            name: "internal_names",
            content: include_str!("./internal_names.puml"),
            parsed: build_internal_names_fsm().expect("Failed to create expected FSM"),
            path,
        }
    }
}
