//! Fixtures for the submachine (`state X : SubName`) parse path.

use crate::{
    error::Result,
    fsm::{Action, Event, TransitionParameters, UmlFsm, UmlFsmBuilder},
};

/// Root FSM whose `Active` state references the `Worker` sub-FSM.
const ROOT: &str = include_str!("./submachine.puml");

/// The `Worker` sub-FSM spliced into `Active`.
const WORKER: &str = include_str!("./worker.puml");

/// The `Task` sub-FSM spliced into `Worker`'s `Working` state (second level of nesting).
const TASK: &str = include_str!("./task.puml");

/// Reference data for a root FSM parsed together with a pool of sub-FSMs.
pub struct SubFsmTestData {
    pub content: &'static str,
    pub subs: &'static [&'static str],
    pub parsed: UmlFsm,
}

impl SubFsmTestData {
    pub fn submachine() -> Self {
        Self {
            content: ROOT,
            subs: &[WORKER, TASK],
            parsed: build_submachine_fsm().expect("Failed to create expected FSM"),
        }
    }
}

fn build_submachine_fsm() -> Result<UmlFsm> {
    let mut builder = UmlFsmBuilder::new("Submachine");

    // `state Active : Worker` expands into a composite `Active` holding the Worker region.
    let active = builder.add_state("Active");
    builder.set_scope(Some(active));
    let working = builder.add_state("Working");
    builder.add_transition(TransitionParameters::Enter { target: "Working" });

    // `state Working : Task` nests a second level: Working holds the Task region.
    builder.set_scope(Some(working));
    builder.add_transition(TransitionParameters::Enter { target: "Running" });
    builder.add_transition(TransitionParameters::Event {
        source: "Running",
        target: "Complete",
        event: Event("Finish".into()),
        action: Some(Action("HandleFinish".into())),
        guard: None,
    });
    builder.set_scope(None);

    // Root region.
    builder.add_transition(TransitionParameters::Enter { target: "Idle" });
    builder.add_transition(TransitionParameters::Event {
        source: "Idle",
        target: "Active",
        event: Event("Start".into()),
        action: None,
        guard: None,
    });
    builder.add_transition(TransitionParameters::Event {
        source: "Active",
        target: "Idle",
        event: Event("Stop".into()),
        action: None,
        guard: None,
    });

    builder.build()
}
