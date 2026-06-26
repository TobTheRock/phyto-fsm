use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "test/actions/enter_exit.puml",
    log_level = "debug"
);

use enter_exit_actions::{IEnterExitActionsActions, IEnterExitActionsEventParams};
use mockall::{Sequence, mock};

mock! {
    Actions {}
    impl IEnterExitActionsActions for Actions {
        fn enter_state_a(&mut self);
        fn exit_state_a(&mut self);
        fn enter_state_c(&mut self);
        fn exit_state_c(&mut self);
        fn enter_state_ca(&mut self);
        fn exit_state_ca(&mut self);
    }
}

impl IEnterExitActionsEventParams for MockActions {
    type GoToAFromAParams = ();
    type GoToBParams = ();
    type GoToAFromBParams = ();
    type GoToCParams = ();
    type GoToCaFromAParams = ();
    type GoToCbFromAParams = ();
    type GoToAFromCParams = ();
    type GoToCbParams = ();
}

struct EnterExitTests {
    actions: MockActions,
    seq: Sequence,
}

impl EnterExitTests {
    fn new() -> Self {
        let mut t = Self {
            actions: MockActions::new(),
            seq: Sequence::new(),
        };
        // ignore: [*] -> A
        t.expect_enter_state_a();
        t
    }
    fn expect_enter_state_a(&mut self) {
        self.actions
            .expect_enter_state_a()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_exit_state_a(&mut self) {
        self.actions
            .expect_exit_state_a()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_enter_state_c(&mut self) {
        self.actions
            .expect_enter_state_c()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_exit_state_c(&mut self) {
        self.actions
            .expect_exit_state_c()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_enter_state_ca(&mut self) {
        self.actions
            .expect_enter_state_ca()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_exit_state_ca(&mut self) {
        self.actions
            .expect_exit_state_ca()
            .returning(|| ())
            .times(1)
            .in_sequence(&mut self.seq);
    }

    fn expect_a_to_c1(&mut self) {
        self.expect_exit_state_a();
        self.expect_enter_state_c();
        self.expect_enter_state_ca();
    }

    fn expect_a_to_c2(&mut self) {
        self.expect_exit_state_a();
        // C2 doesn't have its own enter, so it should call C's enter
        self.expect_enter_state_c();
    }
}

#[test]
fn enter_state_action_called_on_initial_state() {
    let mut actions = MockActions::new();
    actions.expect_enter_state_a().returning(|| ()).times(1);

    let _fsm = enter_exit_actions::start(actions);
}

#[test]
fn exit_state_action_called_when_leaving_state() {
    let mut t = EnterExitTests::new();

    t.expect_exit_state_a();

    let mut fsm = enter_exit_actions::start(t.actions);
    fsm.go_to_b(());
}

#[test]
fn enter_state_action_called_when_entering_state() {
    let mut t = EnterExitTests::new();

    // A -> B
    t.expect_exit_state_a();
    // B -> A
    t.expect_enter_state_a();

    let mut fsm = enter_exit_actions::start(t.actions);
    fsm.go_to_b(());
    fsm.go_to_a_from_b(());
}

#[test]
fn parent_enter_before_substate_enter() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c1();

    let mut fsm = enter_exit_actions::start(t.actions);
    fsm.go_to_ca_from_a(());
}

#[test]
fn substate_exit_before_parent_exit() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c1();

    // C1 -> A
    t.expect_exit_state_ca();
    t.expect_exit_state_c();
    t.expect_enter_state_a();

    let mut fsm = enter_exit_actions::start(t.actions);
    fsm.go_to_ca_from_a(());
    fsm.go_to_a_from_c(());
}

#[test]
fn substate_entry_defaults_to_parent_enter() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c2();

    let mut fsm = enter_exit_actions::start(t.actions);
    fsm.go_to_cb_from_a(());
}

#[test]
fn substate_exit_defaults_to_parent_exit() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c2();
    // C2 -> A
    t.expect_exit_state_c();
    t.expect_enter_state_a();

    let mut fsm = enter_exit_actions::start(t.actions);
    fsm.go_to_cb_from_a(());
    fsm.go_to_a_from_c(());
}

#[test]
fn internal_substate_transition_only_calls_substate_actions() {
    let mut t = EnterExitTests::new();

    t.expect_a_to_c1();
    // C1 -> C2
    t.expect_exit_state_ca();
    t.actions.expect_enter_state_c().never();
    t.actions.expect_exit_state_c().never();

    let mut fsm = enter_exit_actions::start(t.actions);
    fsm.go_to_ca_from_a(());
    fsm.go_to_cb(());
}

#[test]
fn self_transition_calls_exit_state_and_enter() {
    let mut t = EnterExitTests::new();

    t.expect_exit_state_a();
    t.expect_enter_state_a();

    let mut fsm = enter_exit_actions::start(t.actions);
    fsm.go_to_a_from_a(());
}

// TODO internal transitions
