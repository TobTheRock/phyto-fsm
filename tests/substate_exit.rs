/// Test that a substate exiting to `[*]` completes its parent region rather than terminating
/// the whole FSM.
///
/// `Idle --> [*] : Finish` inside `Working` means "Working's region is done": it should fire
/// `Working`'s completion transition (`Working --> Done`), NOT end the machine. Only the
/// top-level `Done --> [*] : Shutdown` terminates the FSM.
///
/// Covers:
/// - A substate `--> [*]` lands on the parent composite's completion target (`Done`)
/// - The FSM stays alive after the region completes
/// - A subsequent top-level `--> [*]` still ends the FSM
use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/exit_states/substate_exit.puml",
    log_level = "debug"
);

use mockall::mock;
use substate_exit::{ISubstateExitActions, ISubstateExitEventParams, NoEventData};

mock! {
    SubstateExitActions {}
    impl ISubstateExitActions for SubstateExitActions {}
}

impl ISubstateExitEventParams for MockSubstateExitActions {
    type PauseParams = NoEventData;
    type FinishParams = NoEventData;
    type ShutdownParams = NoEventData;
}

#[test]
fn substate_exit_completes_region_without_ending_fsm() {
    let mut fsm = substate_exit::start(MockSubstateExitActions::new());
    assert_eq!(
        fsm.active_state(),
        Some(substate_exit::SubstateExitState::WorkingBusy),
        "starts in the composite's initial substate"
    );

    fsm.pause(());
    assert_eq!(
        fsm.active_state(),
        Some(substate_exit::SubstateExitState::WorkingIdle)
    );

    // Idle --[Finish]--> [*]: Working's region completes, firing `Working --> Done`.
    // The FSM must NOT end here — it lands on the completion target.
    fsm.finish(());
    assert_eq!(
        fsm.active_state(),
        Some(substate_exit::SubstateExitState::Done),
        "substate [*] completes the region onto the parent's completion target, FSM stays alive"
    );

    // Done --[Shutdown]--> [*]: the top-level exit ends the FSM.
    fsm.shutdown(());
    assert_eq!(fsm.active_state(), None, "top-level [*] ends the FSM");
}
