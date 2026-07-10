use crate::fsm::{Event, State, TransitionParameters, UmlFsm, UmlFsmBuilder};

use crate::fsm::model::StateId;

#[test]
fn add_substate() {
    let mut builder = builder_with_enter();
    add_state_with_substate(&mut builder, "Parent", "Child", false);
    let fsm = builder.build().unwrap();

    let parent = find_state(&fsm, "Parent");
    assert_eq!(parent.substates().count(), 1);

    let substate = parent.substates().next().unwrap();
    assert_eq!(substate.name(), "Child");
    assert!(!substate.is_enter());
    assert_eq!(substate.parent().unwrap(), parent);
}

#[test]
fn add_substate_same_name_different_parents() {
    let mut builder = builder_with_enter();
    add_state_with_substate(&mut builder, "Parent1", "Child", false);
    add_state_with_substate(&mut builder, "Parent2", "Child", false);
    let fsm = builder.build().unwrap();

    assert_n_times_state(&fsm, "Parent1", 1);
    assert_n_times_state(&fsm, "Parent2", 1);
    assert_n_times_state(&fsm, "Child", 2);
}

#[test]
fn add_substate_enter() {
    let mut builder = builder_with_enter();
    add_state_with_substate(&mut builder, "Parent", "InitialChild", true);
    let fsm = builder.build().unwrap();

    let parent = find_state(&fsm, "Parent");
    let child = parent.substates().next().unwrap();
    assert_eq!(child.name(), "InitialChild");
    assert!(child.is_enter());
}

#[test]
fn add_nested_substates() {
    let mut builder = builder_with_enter();
    let (_, l2) = add_state_with_substate(&mut builder, "Level1", "Level2", false);
    builder.set_scope(Some(l2));
    builder.add_state("Level3");
    let fsm = builder.build().unwrap();

    let level1 = find_state(&fsm, "Level1");
    let level2 = level1.substates().next().unwrap();
    let level3 = level2.substates().next().unwrap();
    assert_eq!(level3.name(), "Level3");
    assert_eq!(level3.parent().unwrap(), level2);
}

#[test]
fn add_substate_transition() {
    let mut builder = builder_with_enter();
    let parent = builder.add_state("Parent");
    builder.set_scope(Some(parent));
    builder.add_transition(TransitionParameters::Enter { target: "A" });
    builder.add_state("B");
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "B",
        event: "E1".into(),
        action: None,
        guard: None,
    });
    let fsm = builder.build().unwrap();

    let parent_state = find_state(&fsm, "Parent");
    let a = parent_state.substates().find(|s| s.name() == "A").unwrap();
    let t = a.transitions().next().unwrap();
    assert_eq!(t.destination().unwrap().name(), "B");
    assert_eq!(t.event(), Some(&Event::from("E1")));
}

#[test]
fn add_substate_transition_same_name_different_parents() {
    let mut builder = builder_with_enter();
    let p1 = builder.add_state("Parent1");
    let p2 = builder.add_state("Parent2");

    builder.set_scope(Some(p1));
    builder.add_transition(TransitionParameters::Enter { target: "A" });
    builder.add_state("B");
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "B",
        event: "E1".into(),
        action: None,
        guard: None,
    });

    builder.set_scope(Some(p2));
    builder.add_transition(TransitionParameters::Enter { target: "A" });
    builder.add_state("B");
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "B",
        event: "E2".into(),
        action: None,
        guard: None,
    });

    let fsm = builder.build().unwrap();

    let parent1 = find_state(&fsm, "Parent1");
    let parent2 = find_state(&fsm, "Parent2");
    let p1_a = parent1.substates().find(|s| s.name() == "A").unwrap();
    let p2_a = parent2.substates().find(|s| s.name() == "A").unwrap();

    assert_eq!(
        p1_a.transitions().next().unwrap().event(),
        Some(&Event::from("E1"))
    );
    assert_eq!(
        p2_a.transitions().next().unwrap().event(),
        Some(&Event::from("E2"))
    );
}

#[test]
fn add_substate_transition_creates_substates() {
    let mut builder = builder_with_enter();
    let parent = builder.add_state("Parent");
    builder.set_scope(Some(parent));
    builder.add_transition(TransitionParameters::Event {
        source: "A",
        target: "B",
        event: "E1".into(),
        action: None,
        guard: None,
    });
    let fsm = builder.build().unwrap();

    assert_eq!(find_state(&fsm, "Parent").substates().count(), 2);
}

fn builder_with_enter() -> UmlFsmBuilder {
    let mut builder = UmlFsmBuilder::new("TestFSM");
    builder.add_transition(TransitionParameters::Enter { target: "Start" });
    builder
}

fn find_state<'a>(fsm: &'a UmlFsm, name: &str) -> State<'a> {
    fsm.states().find(|s| s.name() == name).unwrap()
}

fn assert_n_times_state(fsm: &UmlFsm, name: &str, n: usize) {
    let count = fsm.states().filter(|s| s.name() == name).count();
    assert_eq!(
        count, n,
        "Expected {} states named '{}' found {}",
        n, name, count
    );
}

fn add_state_with_substate(
    builder: &mut UmlFsmBuilder,
    parent: &str,
    child: &str,
    child_enter: bool,
) -> (StateId, StateId) {
    let parent_id = builder.add_state(parent);
    builder.set_scope(Some(parent_id));
    let child_id = if child_enter {
        builder.add_transition(TransitionParameters::Enter { target: child })
    } else {
        builder.add_state(child)
    };
    builder.set_scope(None);
    (parent_id, child_id)
}
