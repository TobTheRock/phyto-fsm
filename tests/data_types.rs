use phyto_fsm::generate_fsm;
generate_fsm!(
    file_path = "../src/test/actions/actions.puml",
    log_level = "debug"
);

use test_fsm::{ITestFsmActions, ITestFsmEventParams};

struct ActionsWithClonableData;
impl ITestFsmEventParams for ActionsWithClonableData {
    type GoToBParams = String;
    type GoToAParams = Vec<i32>;
}
impl ITestFsmActions for ActionsWithClonableData {
    fn handle_go_to_b(&mut self, event: String) {
        println!("Action1 called with event data: {}", event);
    }
    fn handle_go_to_a(&mut self, event: Vec<i32>) {
        println!("Action2 called with event data: {:?}", event);
    }
}
#[test]
fn actions_with_clonable_data() {
    let actions = ActionsWithClonableData;
    let mut fsm = test_fsm::start(actions);
    fsm.go_to_b("Hello FSM".to_string());
    fsm.go_to_a(vec![10, 20, 30]);
}

struct ActionsWithPointers<'a> {
    phantom: std::marker::PhantomData<&'a ()>,
}
impl<'a> ITestFsmEventParams for ActionsWithPointers<'a> {
    type GoToAParams = *const i32;
    type GoToBParams = &'a str;
}
impl<'a> ITestFsmActions for ActionsWithPointers<'a> {
    fn handle_go_to_b(&mut self, event: &'a str) {
        println!("Action1 called with event data: {}", event);
    }
    fn handle_go_to_a(&mut self, event: *const i32) {
        unsafe {
            if !event.is_null() {
                println!("Action2 called with event data: {}", *event);
            } else {
                println!("Action2 called with null pointer");
            }
        }
    }
}

#[test]
fn actions_with_pointers() {
    let actions = ActionsWithPointers {
        phantom: std::marker::PhantomData,
    };
    let mut fsm = test_fsm::start(actions);
    fsm.go_to_b("Pointer Event");
    let number: i32 = 100;
    let number_ptr: *const i32 = &number as *const i32;
    fsm.go_to_a(number_ptr);
}
