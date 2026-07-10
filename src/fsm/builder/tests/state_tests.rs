use crate::fsm::{State, TransitionParameters, UmlFsm, UmlFsmBuilder};

#[test]
fn add_state_creates_simple_state() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "Start" });
    builder.add_state("State1");
    let fsm = builder.build().unwrap();

    assert_eq!(fsm.states().count(), 2);
    let state1 = find_state(&fsm, "State1");
    assert!(!state1.is_enter());
}

#[test]
fn add_state_reuses_existing() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "A" });
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "B",
        event: "E1".into(),
        action: None,
        guard: None,
    });
    builder.add_state("B");
    let fsm = builder.build().unwrap();

    assert_eq!(fsm.states().count(), 2);
}

#[test]
fn enter_state_not_overwritten_by_simple() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "Start" });
    builder.add_state("Start");
    let fsm = builder.build().unwrap();

    let start = find_state(&fsm, "Start");
    assert!(start.is_enter());
}

#[test]
fn simple_state_upgraded_to_enter() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Event {
        source: "Start",
        target: "B",
        event: "E1".into(),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Enter { target: "Start" });
    let fsm = builder.build().unwrap();

    let start = find_state(&fsm, "Start");
    assert!(start.is_enter());
}

fn find_state<'a>(fsm: &'a UmlFsm, name: &str) -> State<'a> {
    fsm.states().find(|s| s.name() == name).unwrap()
}
