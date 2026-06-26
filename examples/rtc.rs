//! # Run-to-Completion (RTC) — External Event Loop
//!
//! Transition actions in phyto-fsm are atomic: the FSM is mutably borrowed for
//! the entire transition, so an action cannot call back into the FSM to trigger
//! another event. Rust's borrow checker enforces this at compile time.
//!
//! When an action needs to trigger a follow-up event, one way is to use a channel to signal
//! the intent and process it in an external event loop after the transition completes.

use std::sync::mpsc;

use phyto_fsm::generate_fsm;
generate_fsm!(file_path = "test/actions/actions.puml");

use test_fsm::{ITestFsmActions, ITestFsmEventParams};

enum FollowUpEvent {
    GoToA(i32),
}

struct MyActions {
    follow_up_tx: mpsc::Sender<FollowUpEvent>,
}

impl ITestFsmEventParams for MyActions {
    type GoToBParams = ();
    type GoToAParams = i32;
}

impl ITestFsmActions for MyActions {
    fn handle_go_to_b(&mut self, _event: ()) {
        println!("Action1: transitioning to StateB, requesting follow-up GoToA");
        self.follow_up_tx.send(FollowUpEvent::GoToA(42)).unwrap();
    }

    fn handle_go_to_a(&mut self, event: i32) {
        println!("Action2: transitioning back to StateA with param={event}");
    }
}

// Action1 (fired during GoToB) signals a GoToA follow-up event through a
// channel. The external loop picks it up and feeds it back into the FSM,
// which then transitions StateB -> StateA and calls Action2.
fn main() {
    let (tx, rx) = mpsc::channel();
    let actions = MyActions { follow_up_tx: tx };
    let mut fsm = test_fsm::start(actions);

    // Trigger GoToB — Action1 fires and enqueues a follow-up GoToA
    fsm.go_to_b(());

    // External event loop: drain follow-up events
    while let Ok(event) = rx.try_recv() {
        match event {
            FollowUpEvent::GoToA(param) => fsm.go_to_a(param),
        }
    }
}
