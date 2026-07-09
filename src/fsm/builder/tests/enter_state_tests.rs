use crate::fsm::{TransitionParameters, UmlFsmBuilder};

#[test]
fn add_enter_state() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("Start");

    let fsm = builder.build().unwrap();
    let enter = fsm.enter_state();
    assert_eq!(enter.name(), "Start");
    assert!(enter.is_enter());
}

#[test]
fn add_enter_state_twice_fails() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("Start");
    builder.add_enter_state("AnotherStart");

    let result = builder.build();
    assert!(result.is_err());
}

#[test]
fn add_enter_state_after_transition() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("B"),
        event: Some("Event".into()),
        action: None,
        guard: None,
    });
    builder.add_enter_state("Start");

    let fsm = builder.build().unwrap();
    let enter = fsm.enter_state();
    assert_eq!(enter.name(), "Start");
    assert!(enter.is_enter());
}

#[test]
fn add_transition_after_enter_state() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("Start");
    builder.add_transition(TransitionParameters {
        source: "A",
        target: Some("B"),
        event: Some("Event".into()),
        action: None,
        guard: None,
    });

    let fsm = builder.build().unwrap();
    let enter = fsm.enter_state();
    assert_eq!(enter.name(), "Start");
    assert!(enter.is_enter());
}

#[test]
fn enter_state_resolves_to_deepest_nested_enter() {
    let mut builder = UmlFsmBuilder::new("TestFSM");

    let root = builder.add_enter_state("RootEnter");
    builder.set_scope(Some(root));
    let nested = builder.add_enter_state("NestedEnter");
    builder.add_state("NestedSimple");

    builder.set_scope(Some(nested));
    builder.add_enter_state("DeepestEnter");
    builder.add_state("DeepestSimple");

    let fsm = builder.build().unwrap();
    assert_eq!(fsm.enter_state().name(), "DeepestEnter");
}

#[test]
fn sets_deepest_enter_state_on_composite() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("RootEnter");
    let root = builder.add_state("Composite");
    builder.set_scope(Some(root));
    let nested = builder.add_enter_state("NestedEnter");
    builder.set_scope(Some(nested));
    builder.add_enter_state("DeepestEnter");
    let fsm = builder.build().unwrap();

    let composite = fsm.states().find(|s| s.name() == "Composite").unwrap();
    assert_eq!(composite.enter_state().name(), "DeepestEnter");
}
