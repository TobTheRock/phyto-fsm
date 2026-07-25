/// A substate exiting to `[*]` fans out over *every* completion transition of its parent region,
/// not just one. `Working` completes via a guarded choice (`[LowBattery] / Cleanup` vs `[Ok]`), so
/// `Idle --> [*] : Finish` lowers to both guarded branches — the taken branch's target and effect
/// win, and the FSM stays alive (only the top-level `Done --> [*]` terminates).
use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/exit_states/guarded_completion.puml",
    log_level = "debug"
);

use guarded_completion::{IGuardedCompletionActions, IGuardedCompletionEventParams, NoEventData};
use mockall::mock;

mock! {
    GuardedCompletionActions {}
    impl IGuardedCompletionActions for GuardedCompletionActions {
        fn low_battery(&self, event: &NoEventData) -> bool;
        fn ok(&self, event: &NoEventData) -> bool;
        fn cleanup(&mut self, event: NoEventData);
    }
}

impl IGuardedCompletionEventParams for MockGuardedCompletionActions {
    type PauseParams = NoEventData;
    type FinishParams = NoEventData;
    type ShutdownParams = NoEventData;
}

#[test]
fn finish_takes_low_battery_completion_branch() {
    let mut actions = MockGuardedCompletionActions::new();
    actions.expect_low_battery().returning(|_| true);
    actions.expect_ok().returning(|_| false);
    actions.expect_cleanup().returning(|_| ()).once();

    let mut fsm = guarded_completion::start(actions);
    fsm.pause(());
    fsm.finish(());
    assert_eq!(
        fsm.active_state(),
        Some(guarded_completion::GuardedCompletionState::Recharge),
        "the [LowBattery] completion target is taken and its Cleanup effect runs"
    );
}

#[test]
fn finish_takes_ok_completion_branch() {
    let mut actions = MockGuardedCompletionActions::new();
    actions.expect_low_battery().returning(|_| false);
    actions.expect_ok().returning(|_| true);
    actions.expect_cleanup().never();

    let mut fsm = guarded_completion::start(actions);
    fsm.pause(());
    fsm.finish(());
    assert_eq!(
        fsm.active_state(),
        Some(guarded_completion::GuardedCompletionState::Done),
        "the [Ok] completion target is taken, no Cleanup"
    );
}
// TODO: allow mixing with unguarded branches
// #[test]
// fn finish_falls_back_to_unguarded_completion_branch() {
//     let mut actions = MockGuardedCompletionActions::new();
//     actions.expect_low_battery().returning(|_| false);
//     actions.expect_ok().returning(|_| false);
//     actions.expect_cleanup().never();
//
//     let mut fsm = guarded_completion::start(actions);
//     fsm.pause(());
//     fsm.finish(());
//     assert_eq!(
//         fsm.active_state(),
//         Some(guarded_completion::GuardedCompletionState::Standby),
//         "both guards fail, so the unguarded default completion is taken"
//     );
// }
