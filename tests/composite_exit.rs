/// Test that a top-level composite region can exit to `[*]`, ending the FSM.
///
/// Covers:
/// - The active substate is torn down (its exit runs) on the way to `[*]`
/// - The composite's own exit action runs
/// - The transition action on the `--> [*]` edge runs
/// - After reaching `[*]` the FSM has no active state and ignores further events
use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/exit_states/composite_exit.puml",
    log_level = "debug"
);

use composite_exit::{ICompositeExitActions, ICompositeExitEventParams, NoEventData};
use mockall::mock;

mock! {
    CompositeExitActions {}
    impl ICompositeExitActions for CompositeExitActions {
        fn cleanup(&mut self);
        fn goodbye(&mut self, event: <MockCompositeExitActions as ICompositeExitEventParams>::ShutdownParams);
    }
}

impl ICompositeExitEventParams for MockCompositeExitActions {
    type PauseParams = NoEventData;
    type ResumeParams = NoEventData;
    type ShutdownParams = NoEventData;
}

#[test]
fn shutdown_from_composite_ends_the_fsm() {
    let mut actions = MockCompositeExitActions::new();
    actions.expect_cleanup().returning(|| ()).times(1);
    actions.expect_goodbye().returning(|_| ()).times(1);

    let mut fsm = composite_exit::start(actions);
    assert_eq!(
        fsm.active_state(),
        Some(composite_exit::CompositeExitState::WorkingBusy),
        "starts in the composite's initial substate"
    );

    // Working --[Shutdown / Goodbye]--> [*]: the substate and composite exit, then the FSM ends
    fsm.shutdown(());
    assert_eq!(fsm.active_state(), None, "reaching [*] ends the FSM");
}

#[test]
fn events_ignored_after_composite_exit() {
    let mut actions = MockCompositeExitActions::new();
    actions.expect_cleanup().returning(|| ()).times(1);
    actions.expect_goodbye().returning(|_| ()).times(1);

    let mut fsm = composite_exit::start(actions);
    fsm.shutdown(());

    // FSM has ended: further events are no-ops
    fsm.pause(());
    assert_eq!(fsm.active_state(), None);
}
