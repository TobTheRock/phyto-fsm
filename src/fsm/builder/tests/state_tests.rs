use crate::fsm::{State, StateType, TransitionParameters, TransitionTarget, UmlFsm, UmlFsmBuilder};

#[test]
fn add_state_creates_simple_state() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_state("State1", StateType::Simple);
    let fsm = builder.build().unwrap();

    assert_eq!(fsm.states().count(), 2);
    let state1 = find_state(&fsm, "State1");
    assert_eq!(state1.state_type(), StateType::Simple);
}

#[test]
fn add_state_reuses_existing() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_state("A", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "A",
        target: TransitionTarget::State("B"),
        event: Some("E1".into()),
        action: None,
        guard: None,
    });
    builder.add_state("B", StateType::Simple);
    let fsm = builder.build().unwrap();

    assert_eq!(fsm.states().count(), 2);
}

#[test]
fn enter_state_not_overwritten_by_simple() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_state("Start", StateType::Simple);
    let fsm = builder.build().unwrap();

    let start = find_state(&fsm, "Start");
    assert_eq!(start.state_type(), StateType::Enter);
}

#[test]
fn simple_state_upgraded_to_enter() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters {
        source: "Start",
        target: TransitionTarget::State("B"),
        event: Some("E1".into()),
        action: None,
        guard: None,
    });
    builder.add_state("Start", StateType::Enter);
    let fsm = builder.build().unwrap();

    let start = find_state(&fsm, "Start");
    assert_eq!(start.state_type(), StateType::Enter);
}

fn find_state<'a>(fsm: &'a UmlFsm, name: &str) -> State<'a> {
    fsm.states().find(|s| s.name() == name).unwrap()
}
