use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/composite_states/composite_states.puml",
    log_level = "debug"
);

use composite_states::{ICompositeStatesActions, ICompositeStatesEventParams};
use mockall::mock;

mock! {
    CompositeStatesActions {}
    impl ICompositeStatesActions for CompositeStatesActions {
        fn handle_go_to_aab(&mut self, event: <MockCompositeStatesActions as ICompositeStatesEventParams>::GoToAabParams);
        fn handle_go_to_ab(&mut self, event: <MockCompositeStatesActions as ICompositeStatesEventParams>::GoToAbParams);
        fn handle_go_to_b(&mut self, event: <MockCompositeStatesActions as ICompositeStatesEventParams>::GoToBParams);
    }
}

impl ICompositeStatesEventParams for MockCompositeStatesActions {
    type GoToAabParams = ();
    type GoToAbParams = ();
    type GoToBParams = ();
}

#[test]
fn should_change_between_nested_substates() {
    let _ = stderrlog::new().verbosity(log::Level::Trace).init();
    let mut actions = MockCompositeStatesActions::new();
    // Only way to reach state AAB is through AAA -> entering substates works
    actions.expect_handle_go_to_aab().returning(|_| ()).times(1);

    let mut fsm = composite_states::start(actions);
    fsm.go_to_aab(());
}

#[test]
fn should_change_between_substates() {
    let mut actions = MockCompositeStatesActions::new();
    // This guaranteses we can exit nested substates, if the parent has a respective transition for
    // the event
    actions.expect_handle_go_to_ab().returning(|_| ()).times(1);
    let mut fsm = composite_states::start(actions);
    fsm.go_to_ab(());
}

#[test]
fn should_change_between_top_level_states() {
    let mut actions = MockCompositeStatesActions::new();
    // This guaranteses we can exit nested substates, if the parent has a respective transition for
    // the event
    actions.expect_handle_go_to_b().returning(|_| ()).times(1);
    let mut fsm = composite_states::start(actions);
    fsm.go_to_b(());
}
