use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/transitions/transitions.puml",
    log_level = "debug"
);

use mockall::mock;
use test_fsm::{ITestFsmActions, ITestFsmEventParams};

mock! {
    TestFsmActions {}
    impl ITestFsmActions for TestFsmActions {
        fn handle_self_transition(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::SelfTransitionParams);
        fn handle_go_to_b(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::SelfTransitionParams);
        fn handle_go_to_b_differently(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::SelfTransitionParams);
    }
}

impl ITestFsmEventParams for MockTestFsmActions {
    type SelfTransitionParams = ();
    type GoToBParams = ();
    type GoToBDifferentlyParams = ();
    type GoToCParams = ();
    type GoToAParams = ();
    type GoToADifferentlyParams = ();
}

#[test]
fn self_transition_action_called() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_handle_self_transition().returning(|_| ()).times(1);
    actions.expect_handle_go_to_b().returning(|_| ()).times(1);
    let mut fsm = test_fsm::start(actions);

    fsm.self_transition(());
    fsm.go_to_b(());
}

#[test]
fn final_state() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_handle_self_transition().times(0);
    actions.expect_handle_go_to_b().returning(|_| ()).times(1);
    let mut fsm = test_fsm::start(actions);

    fsm.go_to_b(());
    fsm.self_transition(());
}

#[test]
fn alternative_transition() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_handle_go_to_b_differently().returning(|_| ()).times(1);
    actions.expect_handle_go_to_b().times(0);
    let mut fsm = test_fsm::start(actions);

    fsm.go_to_b_differently(());
}

#[test]
fn event_list_every_event_transitions() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_handle_go_to_b().returning(|_| ()).times(2);
    let mut fsm = test_fsm::start(actions);

    fsm.go_to_b(());
    fsm.go_to_a(());
    assert_eq!(fsm.active_state(), Some(test_fsm::TestFsmState::StateA));

    fsm.go_to_b(());
    fsm.go_to_a_differently(());
    assert_eq!(fsm.active_state(), Some(test_fsm::TestFsmState::StateA));
}
