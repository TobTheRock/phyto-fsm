use crate::fsm::{StateType, TransitionParameters, TransitionTarget, UmlFsmBuilder};

#[test]
fn add_enter_state() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);

    let fsm = builder.build().unwrap();
    let enter = fsm.enter_state();
    assert_eq!(enter.name(), "Start");
    assert_eq!(enter.state_type(), StateType::Enter);
}

#[test]
fn add_enter_state_twice_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_state("AnotherStart", StateType::Enter);

    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn add_enter_state_after_transition() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters {
        source: "A",
        target: TransitionTarget::State("B"),
        event: Some("Event".into()),
        action: None,
        guard: None,
    });
    builder.add_state("Start", StateType::Enter);

    let fsm = builder.build().unwrap();
    let enter = fsm.enter_state();
    assert_eq!(enter.name(), "Start");
    assert_eq!(enter.state_type(), StateType::Enter);
}

#[test]
fn add_transition_after_enter_state() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_state("Start", StateType::Enter);
    builder.add_transition(TransitionParameters {
        source: "A",
        target: TransitionTarget::State("B"),
        event: Some("Event".into()),
        action: None,
        guard: None,
    });

    let fsm = builder.build().unwrap();
    let enter = fsm.enter_state();
    assert_eq!(enter.name(), "Start");
    assert_eq!(enter.state_type(), StateType::Enter);
}

#[test]
fn enter_state_resolves_to_deepest_nested_enter() {
    let mut builder = UmlFsmBuilder::new("TestFSM");

    let root = builder.add_state("RootEnter", StateType::Enter);
    builder.set_scope(Some(root));
    let nested = builder.add_state("NestedEnter", StateType::Enter);
    builder.add_state("NestedSimple", StateType::Simple);

    builder.set_scope(Some(nested));
    builder.add_state("DeepestEnter", StateType::Enter);
    builder.add_state("DeepestSimple", StateType::Simple);

    let fsm = builder.build().unwrap();
    assert_eq!(fsm.enter_state().name(), "DeepestEnter");
}

#[test]
fn sets_deepest_enter_state_on_composite() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_state("RootEnter", StateType::Enter);
    let root = builder.add_state("Composite", StateType::Simple);
    builder.set_scope(Some(root));
    let nested = builder.add_state("NestedEnter", StateType::Enter);
    builder.set_scope(Some(nested));
    builder.add_state("DeepestEnter", StateType::Enter);
    let fsm = builder.build().unwrap();

    let composite = fsm.states().find(|s| s.name() == "Composite").unwrap();
    assert_eq!(composite.enter_state().name(), "DeepestEnter");
}
