use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/actions/actions.puml",
    log_level = "debug"
);

use mockall::{mock, predicate};
use test_fsm::{ITestFsmActions, ITestFsmEventParams};

mock! {
    TestFsmActions {}
    impl ITestFsmActions for TestFsmActions {
        fn handle_go_to_b(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::GoToBParams);
        fn handle_go_to_a(&mut self, event: <MockTestFsmActions as ITestFsmEventParams>::GoToAParams);
    }
}

impl ITestFsmEventParams for MockTestFsmActions {
    type GoToBParams = ();
    type GoToAParams = i32;
}

#[test]
fn actions_are_called_when_transitioning() {
    let param_for_handle_go_to_a = 7;
    let mut actions = MockTestFsmActions::new();

    actions.expect_handle_go_to_b().returning(|_| ()).times(1);
    actions
        .expect_handle_go_to_a()
        .with(predicate::eq(param_for_handle_go_to_a))
        .returning(|_| ())
        .times(1);
    let mut fsm = test_fsm::start(actions);
    fsm.go_to_b(());
    fsm.go_to_a(param_for_handle_go_to_a);
}

#[test]
fn no_action_called_when_no_transition() {
    let mut actions = MockTestFsmActions::new();
    actions.expect_handle_go_to_b().times(0);
    actions.expect_handle_go_to_a().times(0);
    let mut fsm = test_fsm::start(actions);
    // Trigger event that does not cause a transition from the initial state
    fsm.go_to_a(42);
}
