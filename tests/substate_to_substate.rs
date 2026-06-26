use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/composite_states/substate_to_substate.puml",
    log_level = "debug"
);

use mockall::mock;
use substate_to_substate::{ISubstateToSubstateActions, ISubstateToSubstateEventParams};

mock! {
    SubstateToSubstateActions {}
    impl ISubstateToSubstateActions for SubstateToSubstateActions {
        fn handle_go_to_ba(&mut self, event: <MockSubstateToSubstateActions as ISubstateToSubstateEventParams>::GoToBaParams);
        fn handle_go_to_bb(&mut self, event: <MockSubstateToSubstateActions as ISubstateToSubstateEventParams>::GoToBbParams);
    }
}

impl ISubstateToSubstateEventParams for MockSubstateToSubstateActions {
    type GoToBaParams = ();
    type GoToBbParams = ();
}

#[test]
fn should_transition_from_substate_to_substate_across_parents() {
    // let _ = stderrlog::new().verbosity(log::Level::Trace).init();
    let mut actions = MockSubstateToSubstateActions::new();
    // Starting in A::AA, transition to B::BA should trigger handle_go_to_ba
    actions.expect_handle_go_to_ba().returning(|_| ()).times(1);

    let mut fsm = substate_to_substate::start(actions);
    fsm.go_to_ba(());
}

#[test]
fn should_transition_within_substate() {
    let mut actions = MockSubstateToSubstateActions::new();
    // First transition AA -> BA, then BA -> BB
    actions.expect_handle_go_to_ba().returning(|_| ()).times(1);
    actions.expect_handle_go_to_bb().returning(|_| ()).times(1);

    let mut fsm = substate_to_substate::start(actions);
    fsm.go_to_ba(());
    fsm.go_to_bb(());
}
