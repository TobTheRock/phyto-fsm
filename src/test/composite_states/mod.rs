use crate::{
    error::Result,
    fsm::{Action, Event, StateType, TransitionParameters, UmlFsm, UmlFsmBuilder},
    test::{FsmTestData, utils::get_adjacent_file_path},
};

fn build_composite_states_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("Composite States");

    // Root level
    let state_a = builder.add_state("StateA", StateType::Enter);
    builder.add_state("StateB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "StateA",
        target: Some("StateB"),
        event: Some(Event("GoToB".into())),
        action: Some(Action("HandleGoToB".into())),
        guard: None,
    });

    // StateA children
    builder.set_scope(Some(state_a));
    let state_aa = builder.add_state("StateAA", StateType::Enter);
    builder.add_state("StateAB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "StateAA",
        target: Some("StateAB"),
        event: Some(Event("GoToAB".into())),
        action: Some(Action("HandleGoToAB".into())),
        guard: None,
    });
    // StateAA children
    builder.set_scope(Some(state_aa));
    builder.add_state("StateAAA", StateType::Enter);
    builder.add_state("StateAAB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "StateAAA",
        target: Some("StateAAB"),
        event: Some(Event("GoToAAB".into())),
        action: Some(Action("HandleGoToAAB".into())),
        guard: None,
    });

    builder.build()
}

fn build_substate_to_substate_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("Substate To Substate");

    // Root level
    let state_a = builder.add_state("StateA", StateType::Enter);
    let state_b = builder.add_state("StateB", StateType::Simple);

    // StateA's children
    builder.set_scope(Some(state_a));
    builder.add_state("StateAA", StateType::Enter);

    // StateB's children
    builder.set_scope(Some(state_b));
    builder.add_state("StateBA", StateType::Simple);
    builder.add_state("StateBB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "StateBA",
        target: Some("StateBB"),
        event: Some(Event("GoToBB".into())),
        action: Some(Action("HandleGoToBB".into())),
        guard: None,
    });

    // Substate to substate transition (defined at root level but references substates)
    builder.set_scope(None);
    builder.add_transition(TransitionParameters {
        source: "StateAA",
        target: Some("StateBA"),
        event: Some(Event("GoToBA".into())),
        action: Some(Action("HandleGoToBA".into())),
        guard: None,
    });

    builder.build()
}

fn build_same_name_substates_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("Same Name Substates");

    // Root level
    let parent_a = builder.add_state("ParentA", StateType::Enter);
    let parent_b = builder.add_state("ParentB", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "ParentA",
        target: Some("ParentB"),
        event: Some(Event("GoToParentB".into())),
        action: None,
        guard: None,
    });

    // ParentA children
    builder.set_scope(Some(parent_a));
    builder.add_state("Inner", StateType::Enter);
    builder.add_state("Other", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "Inner",
        target: Some("Other"),
        event: Some(Event("GoToOther".into())),
        action: None,
        guard: None,
    });

    // ParentB children
    builder.set_scope(Some(parent_b));
    builder.add_state("Inner", StateType::Enter);
    builder.add_state("Other", StateType::Simple);
    builder.add_transition(TransitionParameters {
        source: "Inner",
        target: Some("Other"),
        event: Some(Event("GoToOther".into())),
        action: None,
        guard: None,
    });

    builder.build()
}

impl FsmTestData {
    pub fn composite_states() -> Self {
        let path = get_adjacent_file_path(file!(), "composite_states.puml");
        Self {
            name: "composite_states",
            content: include_str!("./composite_states.puml"),
            parsed: build_composite_states_fsm().expect("Failed to create FSM for testing"),
            path,
        }
    }

    pub fn same_name_substates() -> Self {
        let path = get_adjacent_file_path(file!(), "same_name_substates.puml");
        Self {
            name: "same_name_substates",
            content: include_str!("./same_name_substates.puml"),
            parsed: build_same_name_substates_fsm().expect("Failed to create FSM for testing"),
            path,
        }
    }

    pub fn substate_to_substate() -> Self {
        let path = get_adjacent_file_path(file!(), "substate_to_substate.puml");
        Self {
            name: "substate_to_substate",
            content: include_str!("./substate_to_substate.puml"),
            parsed: build_substate_to_substate_fsm().expect("Failed to create FSM for testing"),
            path,
        }
    }
}
