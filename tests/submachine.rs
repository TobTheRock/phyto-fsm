use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/submachine/submachine.puml",
    sub_fsms = [
        "../src/test/submachine/worker.puml",
        "../src/test/submachine/task.puml",
    ],
    log_level = "debug"
);

use mockall::mock;
use submachine::{ISubmachineActions, ISubmachineEventParams};

mock! {
    SubmachineActions {}
    impl ISubmachineActions for SubmachineActions {
        fn handle_finish(&mut self, event: <MockSubmachineActions as ISubmachineEventParams>::FinishParams);
    }
}

impl ISubmachineEventParams for MockSubmachineActions {
    type StartParams = ();
    type StopParams = ();
    type FinishParams = ();
}

#[test]
fn should_enter_submachine_and_handle_its_event() {
    let mut actions = MockSubmachineActions::new();
    // Entering `Active` enters the `Worker` submachine at its initial state (Working);
    // `Finish` then transitions inside the submachine and runs its action.
    actions.expect_handle_finish().returning(|_| ()).times(1);

    let mut fsm = submachine::start(actions);

    fsm.start(()); // Idle -> Active -> Worker(Working) -> Task(Running)
    fsm.finish(()); // Running -> Complete, in the innermost (Task) submachine
}
