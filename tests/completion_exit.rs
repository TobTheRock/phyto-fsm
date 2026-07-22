/// Test an event-less ("completion") exit: `Done --> [*]` with no trigger fires as soon as
/// `Done` is reached, ending the FSM.
///
/// Covers:
/// - Reaching `Done` immediately follows its triggerless `--> [*]` to the final state
/// - `Done`'s exit action runs as it is torn down
/// - After the completion exit the FSM has no active state
use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/exit_states/completion_exit.puml",
    log_level = "debug"
);

use completion_exit::{ICompletionExitActions, ICompletionExitEventParams, NoEventData};
use mockall::mock;

mock! {
    CompletionExitActions {}
    impl ICompletionExitActions for CompletionExitActions {
        fn cleanup(&mut self);
    }
}

impl ICompletionExitEventParams for MockCompletionExitActions {
    type FinishParams = NoEventData;
}

#[test]
fn completion_exit_ends_the_fsm() {
    let mut actions = MockCompletionExitActions::new();
    actions.expect_cleanup().returning(|| ()).times(1);

    let mut fsm = completion_exit::start(actions);
    assert_eq!(
        fsm.active_state(),
        Some(completion_exit::CompletionExitState::Active)
    );

    // Finish -> Done, whose event-less `--> [*]` fires immediately, ending the FSM
    fsm.finish(());
    assert_eq!(fsm.active_state(), None, "the completion exit ends the FSM");
}

#[test]
fn events_ignored_after_completion_exit() {
    let mut actions = MockCompletionExitActions::new();
    actions.expect_cleanup().returning(|| ()).times(1);

    let mut fsm = completion_exit::start(actions);
    fsm.finish(());

    // FSM has ended: further events are no-ops, exit action does not run again
    fsm.finish(());
    assert_eq!(fsm.active_state(), None);
}
