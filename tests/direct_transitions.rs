use phyto_fsm::generate_fsm;
generate_fsm!(file_path = "test/transitions/direct_transitions.puml");

use direct_transitions::{IDirectTransitionsActions, IDirectTransitionsEventParams};
use mockall::mock;

mock! {
    Actions {}
    impl IDirectTransitionsActions for Actions {
        // Direct transition actions (no event params)
        fn action_to_b(&mut self);
        fn action_to_c(&mut self);
        // Direct transition guards (no event params)
        fn can_go_to_c(&self) -> bool;
        fn can_go_to_d(&self) -> bool;
        // Enter actions
        fn enter_state_d(&mut self);
    }
}

impl IDirectTransitionsEventParams for MockActions {
    type GoToAParams = ();
}

#[test]
fn direct_transition_fires_on_start() {
    let mut actions = MockActions::new();

    // On start, StateA is entered, then direct transition to StateB fires
    actions.expect_action_to_b().returning(|| ()).times(1);
    // No direct transitions from StateB match (guards return false)
    actions.expect_can_go_to_c().returning(|| false);
    actions.expect_can_go_to_d().returning(|| false);

    let _fsm = direct_transitions::start(actions);
}

#[test]
fn guarded_direct_transition_fires_when_guard_passes() {
    let mut actions = MockActions::new();

    // On start: direct transition StateA -> StateB
    actions.expect_action_to_b().returning(|| ()).times(1);
    // StateB: CanGoToC returns true -> direct transition to StateC with action
    actions.expect_can_go_to_c().returning(|| true);
    actions.expect_action_to_c().returning(|| ()).times(1);

    let _fsm = direct_transitions::start(actions);
}

#[test]
fn guarded_direct_transition_to_state_with_enter_action() {
    let mut actions = MockActions::new();

    // On start: direct transition StateA -> StateB
    actions.expect_action_to_b().returning(|| ()).times(1);
    // StateB: CanGoToC false, CanGoToD true -> direct transition to StateD
    actions.expect_can_go_to_c().returning(|| false);
    actions.expect_can_go_to_d().returning(|| true);
    actions.expect_enter_state_d().returning(|| ()).times(1);

    let _fsm = direct_transitions::start(actions);
}

#[test]
fn direct_transition_after_event() {
    let mut actions = MockActions::new();

    // direct transition StateA -> StateB on start and after GoToA avent
    actions.expect_action_to_b().returning(|| ()).times(2);
    actions.expect_can_go_to_c().returning(|| false);
    actions.expect_can_go_to_d().returning(|| false);

    let mut fsm = direct_transitions::start(actions);

    // Event-based transition back to StateA, which triggers direct to StateB again
    fsm.go_to_a(());
}
