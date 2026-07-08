/// Test that an event-triggered transition to the top-level final state `[*]` ends the FSM.
///
/// Covers:
/// - The exit action of the state being left runs on the way to `[*]`
/// - The transition action on the `--> [*]` edge runs
/// - After reaching `[*]` the FSM has no active state and ignores further events
use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/exit_states/exit_states.puml",
    log_level = "debug"
);

use exit_states::{IExitStatesActions, IExitStatesEventParams, NoEventData};
use mockall::mock;

mock! {
    ExitStatesActions {}
    impl IExitStatesActions for ExitStatesActions {
        fn cleanup(&mut self);
        fn goodbye(&mut self, event: <MockExitStatesActions as IExitStatesEventParams>::ShutdownParams);
    }
}

impl IExitStatesEventParams for MockExitStatesActions {
    type WorkParams = NoEventData;
    type ShutdownParams = NoEventData;
}

#[test]
fn shutdown_ends_the_fsm() {
    let mut actions = MockExitStatesActions::new();
    actions.expect_cleanup().returning(|| ()).times(1);
    actions.expect_goodbye().returning(|_| ()).times(1);

    let mut fsm = exit_states::start(actions);
    assert_eq!(
        fsm.active_state(),
        Some(exit_states::ExitStatesState::Active)
    );

    // Active --[Shutdown / Goodbye]--> [*]: exit action runs, then the FSM ends
    fsm.shutdown(());
    assert_eq!(fsm.active_state(), None, "reaching [*] ends the FSM");
}

#[test]
fn events_ignored_after_exit() {
    let mut actions = MockExitStatesActions::new();
    actions.expect_cleanup().returning(|| ()).times(1);
    actions.expect_goodbye().returning(|_| ()).times(1);

    let mut fsm = exit_states::start(actions);
    fsm.shutdown(());

    // FSM has ended: further events are no-ops, no actions fire
    fsm.work(());
    assert_eq!(fsm.active_state(), None);
}
