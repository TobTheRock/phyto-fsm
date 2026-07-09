use itertools::Itertools;

use crate::fsm::{Event, TransitionParameters, UmlFsmBuilder};

#[test]
fn add_deferred_event_to_state() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("Start");
    builder.add_deferred_event("Start", Event::from("MyEvent"));
    let fsm = builder.build().unwrap();

    let start = fsm.states().find(|s| s.name() == "Start").unwrap();
    let deferred: Vec<_> = start.deferred_events().collect();
    assert_eq!(deferred, vec![&Event::from("MyEvent")]);
}

#[test]
fn state_without_deferred_events_returns_empty() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("Start");
    let fsm = builder.build().unwrap();

    let start = fsm.states().find(|s| s.name() == "Start").unwrap();
    assert_eq!(start.deferred_events().count(), 0);
}

#[test]
fn multiple_deferred_events_on_same_state() {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_enter_state("Start");
    builder.add_deferred_event("Start", Event::from("EventA"));
    builder.add_deferred_event("Start", Event::from("EventB"));
    let fsm = builder.build().unwrap();

    let start = fsm.states().find(|s| s.name() == "Start").unwrap();
    let deferred: Vec<_> = start.deferred_events().collect();
    assert_eq!(
        deferred,
        vec![&Event::from("EventA"), &Event::from("EventB")]
    );
}

#[test]
fn inherited_deferred_events() {
    let mut builder = UmlFsmBuilder::new("test");
    let parent_name = "Parent";
    let parent_defer: Event = "Defer1".into();
    let parent = builder.add_enter_state(parent_name);
    builder.add_deferred_event(parent_name, parent_defer.clone());

    builder.set_scope(Some(parent));
    let child_defer: Event = "Defer2".into();
    let child_name = "Child";
    builder.add_state(child_name);
    builder.add_deferred_event(child_name, child_defer.clone());

    let fsm = builder.build().unwrap();
    dbg!(&fsm);

    let parent = fsm.enter_state();
    let child = parent.substates().next().unwrap();

    assert_eq!(
        parent.deferred_events().cloned().collect_vec(),
        std::slice::from_ref(&parent_defer)
    );
    assert_eq!(
        child.deferred_events().cloned().collect_vec(),
        [child_defer, parent_defer]
    );
}

#[test]
fn child_transition_overrides_parent_deferred_event() {
    let mut builder = UmlFsmBuilder::new("test");
    let parent = builder.add_enter_state("Parent");
    builder.add_deferred_event("Parent", Event::from("Evt"));

    builder.set_scope(Some(parent));
    builder.add_state("Child");
    builder.add_transition(TransitionParameters {
        source: "Child",
        target: Some("Parent"),
        event: Some("Evt".into()),
        action: None,
        guard: None,
    });

    let fsm = builder.build().unwrap();
    let child = fsm.enter_state().substates().next().unwrap();

    assert_eq!(child.deferred_events().count(), 0);
}

#[test]
fn multi_level_inheritance() {
    let mut builder = UmlFsmBuilder::new("test");
    let grandparent = builder.add_enter_state("Grandparent");
    builder.add_deferred_event("Grandparent", Event::from("Evt"));

    builder.set_scope(Some(grandparent));
    let parent = builder.add_state("Parent");

    builder.set_scope(Some(parent));
    builder.add_state("Child");

    let fsm = builder.build().unwrap();
    let grandparent_state = fsm.enter_state();
    let parent_state = grandparent_state.substates().next().unwrap();
    let child_state = parent_state.substates().next().unwrap();

    assert_eq!(
        grandparent_state.deferred_events().cloned().collect_vec(),
        [Event::from("Evt")]
    );
    assert_eq!(
        parent_state.deferred_events().cloned().collect_vec(),
        [Event::from("Evt")]
    );
    assert_eq!(
        child_state.deferred_events().cloned().collect_vec(),
        [Event::from("Evt")]
    );
}

#[test]
fn multi_level_override_breaks_chain() {
    let mut builder = UmlFsmBuilder::new("test");
    let grandparent = builder.add_enter_state("Grandparent");
    builder.add_deferred_event("Grandparent", Event::from("Evt"));

    builder.set_scope(Some(grandparent));
    let parent = builder.add_state("Parent");
    // Parent handles the event via transition, breaking inheritance
    builder.add_transition(TransitionParameters {
        source: "Parent",
        target: Some("Grandparent"),
        event: Some("Evt".into()),
        action: None,
        guard: None,
    });

    builder.set_scope(Some(parent));
    builder.add_state("Child");

    let fsm = builder.build().unwrap();
    let grandparent_state = fsm.enter_state();
    let parent_state = grandparent_state.substates().next().unwrap();
    let child_state = parent_state.substates().next().unwrap();

    assert_eq!(
        grandparent_state.deferred_events().cloned().collect_vec(),
        [Event::from("Evt")]
    );
    assert_eq!(parent_state.deferred_events().count(), 0);
    assert_eq!(child_state.deferred_events().count(), 0);
}

#[test]
fn partial_override_of_multiple_events() {
    let mut builder = UmlFsmBuilder::new("test");
    let parent = builder.add_enter_state("Parent");
    builder.add_deferred_event("Parent", Event::from("EvtA"));
    builder.add_deferred_event("Parent", Event::from("EvtB"));

    builder.set_scope(Some(parent));
    builder.add_state("Child");
    // Child handles EvtA, so only EvtB should be inherited
    builder.add_transition(TransitionParameters {
        source: "Child",
        target: Some("Parent"),
        event: Some("EvtA".into()),
        action: None,
        guard: None,
    });

    let fsm = builder.build().unwrap();
    let child = fsm.enter_state().substates().next().unwrap();

    assert_eq!(
        child.deferred_events().cloned().collect_vec(),
        [Event::from("EvtB")]
    );
}

#[test]
fn no_duplicate_when_child_also_defers_same_event() {
    let mut builder = UmlFsmBuilder::new("test");
    let parent = builder.add_enter_state("Parent");
    builder.add_deferred_event("Parent", Event::from("Evt"));

    builder.set_scope(Some(parent));
    builder.add_state("Child");
    builder.add_deferred_event("Child", Event::from("Evt"));

    let fsm = builder.build().unwrap();
    let child = fsm.enter_state().substates().next().unwrap();

    assert_eq!(
        child.deferred_events().cloned().collect_vec(),
        [Event::from("Evt")]
    );
}
